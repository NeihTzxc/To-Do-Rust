use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};

use crate::domain::auth::SyncStatusState;
use crate::domain::task::{CreateTaskInput, Task, TaskStatus, UpdateTaskInput};
use crate::storage::TaskRepository;
use crate::sync::auth::SupabaseAuthClient;

#[derive(Clone)]
pub struct SyncEngine {
    repo: Arc<dyn TaskRepository>,
    auth_client: SupabaseAuthClient,
    state: Arc<Mutex<SyncStatusState>>,
    notify_tx: mpsc::Sender<()>,
    app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
}

impl SyncEngine {
    pub fn new(repo: Arc<dyn TaskRepository>, auth_client: SupabaseAuthClient) -> (Self, mpsc::Receiver<()>) {
        let (notify_tx, notify_rx) = mpsc::channel(100);

        let initial_state = SyncStatusState {
            is_logged_in: false,
            user_email: None,
            state: "idle".to_string(),
            pending_count: 0,
            last_synced_at: None,
            error_message: None,
        };

        let engine = Self {
            repo,
            auth_client,
            state: Arc::new(Mutex::new(initial_state)),
            notify_tx,
            app_handle: Arc::new(Mutex::new(None)),
        };

        (engine, notify_rx)
    }

    pub async fn set_app_handle(&self, handle: tauri::AppHandle) {
        let mut lock = self.app_handle.lock().await;
        *lock = Some(handle);
    }

    pub fn trigger_sync(&self) {
        let _ = self.notify_tx.try_send(());
    }

    pub async fn get_status(&self) -> SyncStatusState {
        let mut status = self.state.lock().await.clone();
        if let Ok(count) = self.repo.count_pending_tasks().await {
            status.pending_count = count;
        }
        if let Ok(user) = self.repo.get_current_user().await {
            status.is_logged_in = user.is_some();
            status.user_email = user.map(|u| u.email);
        }
        status
    }

    async fn get_auth_credentials(&self) -> Option<(String, String)> {
        let user = self.repo.get_current_user().await.ok().flatten()?;
        if !self.auth_client.is_configured() {
            return None;
        }
        let access_token = self.auth_client.get_valid_access_token(&self.repo).await.ok().flatten()?;
        Some((user.user_id, access_token))
    }

    /// Lấy danh sách task hiển thị cho người dùng:
    /// ONLINE MODE: Nếu có kết nối internet và user đã đăng nhập, luôn lấy trực tiếp từ Supabase DB.
    /// Dữ liệu lấy về từ Supabase đồng thời được lưu vào SQLite làm cache.
    /// Nếu mất kết nối internet (Offline) hoặc chưa đăng nhập: Fallback đọc từ SQLite cục bộ.
    pub async fn get_tasks_online_or_offline(&self) -> Result<Vec<Task>> {
        let user = match self.repo.get_current_user().await {
            Ok(Some(u)) => u,
            _ => return self.repo.list_active_tasks().await,
        };

        if !self.auth_client.is_configured() {
            return self.repo.list_active_tasks().await;
        }

        let access_token = match self.auth_client.get_valid_access_token(&self.repo).await {
            Ok(Some(token)) => token,
            _ => return self.repo.list_active_tasks().await,
        };

        // Nếu có task pending ở local (tạo lúc offline), đẩy lên Supabase trước
        let _ = self.push_pending_tasks(&user.user_id, &access_token).await;

        // Truy vấn trực tiếp từ Supabase PostgREST (Online Mode)
        let endpoint = format!(
            "{}/rest/v1/tasks?user_id=eq.{}&deleted_at=is.null",
            self.auth_client.url(),
            user.user_id
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_default();

        let resp_res = client
            .get(&endpoint)
            .header("apikey", self.auth_client.anon_key())
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await;

        let resp = match resp_res {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let err_text = r.text().await.unwrap_or_default();
                eprintln!("[Online Mode] PostgREST trả lỗi: {}, fallback sang SQLite", err_text);
                return self.repo.list_active_tasks().await;
            }
            Err(e) => {
                eprintln!("[Online Mode] Lỗi kết nối Supabase: {:?}, fallback sang SQLite", e);
                let mut st = self.state.lock().await;
                st.state = "offline".to_string();
                self.emit_sync_status(&st).await;
                return self.repo.list_active_tasks().await;
            }
        };

        let remote_tasks: Vec<Task> = match resp.json().await {
            Ok(tasks) => tasks,
            Err(e) => {
                eprintln!("[Online Mode] Lỗi parse JSON từ Supabase: {:?}, fallback sang SQLite", e);
                return self.repo.list_active_tasks().await;
            }
        };

        // Cập nhật dữ liệu từ Supabase vào SQLite cache
        let mut max_updated: i64 = 0;
        let remote_ids: HashSet<String> = remote_tasks.iter().map(|t| t.id.clone()).collect();

        for mut task in remote_tasks.clone() {
            if task.updated_at > max_updated {
                max_updated = task.updated_at;
            }
            if task.user_id.is_none() {
                task.user_id = Some(user.user_id.clone());
            }
            let _ = self.repo.upsert_pulled_task(&task).await;
        }

        if max_updated > 0 {
            let meta_key = format!("last_pulled_at_{}", user.user_id);
            let _ = self.repo.set_sync_meta(&meta_key, &max_updated.to_string()).await;
        }

        // Dọn dẹp cache: nếu task trong SQLite đã synced nhưng không còn trên Supabase -> soft-delete trong cache
        if let Ok(local_tasks) = self.repo.list_active_tasks().await {
            for local_task in local_tasks {
                if local_task.sync_status == "synced" && !remote_ids.contains(&local_task.id) {
                    let _ = self.repo.soft_delete_task(&local_task.id).await;
                    let _ = self.repo.mark_tasks_synced(&[local_task.id]).await;
                }
            }
        }

        // Sắp xếp danh sách trả về theo thứ tự chuẩn của ứng dụng
        let mut sorted_tasks = remote_tasks;
        sorted_tasks.sort_by(|a, b| {
            let done_a = if a.status.is_done() { 1 } else { 0 };
            let done_b = if b.status.is_done() { 1 } else { 0 };
            done_a.cmp(&done_b)
                .then_with(|| b.priority.cmp(&a.priority))
                .then_with(|| b.created_at.cmp(&a.created_at))
        });

        // Cập nhật trạng thái sync thành công
        let now_ms = Utc::now().timestamp_millis();
        let pending = self.repo.count_pending_tasks().await.unwrap_or(0);
        {
            let mut st = self.state.lock().await;
            st.is_logged_in = true;
            st.user_email = Some(user.email);
            st.state = "synced".to_string();
            st.last_synced_at = Some(now_ms);
            st.pending_count = pending;
            st.error_message = None;
            self.emit_sync_status(&st).await;
        }

        Ok(sorted_tasks)
    }

    /// Thêm mới task: Ghi trực tiếp lên Supabase nếu online, lưu SQLite làm offline cache
    pub async fn create_task_online(&self, input: CreateTaskInput) -> Result<Task> {
        let mut task = self.repo.create_task(input).await?;
        if let Ok(()) = self.online_post_task(&task).await {
            task.sync_status = "synced".to_string();
        } else {
            self.trigger_sync();
        }
        Ok(task)
    }

    /// Cập nhật task: Cập nhật trực tiếp lên Supabase nếu online, lưu SQLite làm offline cache
    pub async fn update_task_online(&self, input: UpdateTaskInput) -> Result<Task> {
        let mut task = self.repo.update_task(input).await?;
        let payload = json!({
            "title": task.title,
            "description": task.description,
            "status": task.status.as_str(),
            "priority": task.priority.as_i64(),
            "due_date": task.due_date,
            "start_time": task.start_time,
            "end_time": task.end_time,
            "updated_at": task.updated_at
        });
        if let Ok(()) = self.online_patch_task(&task.id, &payload).await {
            task.sync_status = "synced".to_string();
        } else {
            self.trigger_sync();
        }
        Ok(task)
    }

    /// Cập nhật lịch xếp việc: Cập nhật trực tiếp lên Supabase nếu online
    pub async fn update_schedule_online(
        &self,
        id: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Task> {
        let mut task = self.repo.update_schedule(id, start_time, end_time).await?;
        let payload = json!({
            "start_time": task.start_time,
            "end_time": task.end_time,
            "updated_at": task.updated_at
        });
        if let Ok(()) = self.online_patch_task(&task.id, &payload).await {
            task.sync_status = "synced".to_string();
        } else {
            self.trigger_sync();
        }
        Ok(task)
    }

    /// Đổi trạng thái task: Cập nhật trực tiếp lên Supabase nếu online
    pub async fn change_status_online(&self, id: &str, status: TaskStatus) -> Result<Task> {
        let mut task = self.repo.change_status(id, status).await?;
        let payload = json!({
            "status": task.status.as_str(),
            "updated_at": task.updated_at
        });
        if let Ok(()) = self.online_patch_task(&task.id, &payload).await {
            task.sync_status = "synced".to_string();
        } else {
            self.trigger_sync();
        }
        Ok(task)
    }

    /// Toggle trạng thái task (todo <-> done): Cập nhật trực tiếp lên Supabase nếu online
    pub async fn toggle_status_online(&self, id: &str) -> Result<Task> {
        let mut task = self.repo.toggle_status(id).await?;
        let payload = json!({
            "status": task.status.as_str(),
            "updated_at": task.updated_at
        });
        if let Ok(()) = self.online_patch_task(&task.id, &payload).await {
            task.sync_status = "synced".to_string();
        } else {
            self.trigger_sync();
        }
        Ok(task)
    }

    /// Xóa mềm task: Cập nhật deleted_at trực tiếp lên Supabase nếu online
    pub async fn delete_task_online(&self, id: &str) -> Result<()> {
        let now = Utc::now().timestamp_millis();
        self.repo.soft_delete_task(id).await?;
        let payload = json!({
            "deleted_at": now,
            "updated_at": now
        });
        if let Err(_) = self.online_patch_task(id, &payload).await {
            self.trigger_sync();
        }
        Ok(())
    }

    async fn online_post_task(&self, task: &Task) -> Result<()> {
        let (user_id, access_token) = match self.get_auth_credentials().await {
            Some(c) => c,
            None => anyhow::bail!("Offline hoặc chưa đăng nhập"),
        };

        let endpoint = format!("{}/rest/v1/tasks?on_conflict=id", self.auth_client.url());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default();

        let payload = json!({
            "id": task.id,
            "user_id": user_id,
            "title": task.title,
            "description": task.description,
            "status": task.status.as_str(),
            "priority": task.priority.as_i64(),
            "due_date": task.due_date,
            "start_time": task.start_time,
            "end_time": task.end_time,
            "created_at": task.created_at,
            "updated_at": task.updated_at,
            "deleted_at": task.deleted_at
        });

        let resp = client
            .post(&endpoint)
            .header("apikey", self.auth_client.anon_key())
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Prefer", "resolution=merge-duplicates,return=minimal")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Không thể gửi task lên Supabase")?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Supabase POST error: {}", err);
        }

        self.repo.mark_tasks_synced(&[task.id.clone()]).await?;
        Ok(())
    }

    async fn online_patch_task(&self, id: &str, payload: &serde_json::Value) -> Result<()> {
        let (user_id, access_token) = match self.get_auth_credentials().await {
            Some(c) => c,
            None => anyhow::bail!("Offline hoặc chưa đăng nhập"),
        };

        let endpoint = format!(
            "{}/rest/v1/tasks?id=eq.{}&user_id=eq.{}",
            self.auth_client.url(),
            id,
            user_id
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default();

        let resp = client
            .patch(&endpoint)
            .header("apikey", self.auth_client.anon_key())
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Prefer", "return=minimal")
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .context("Không thể cập nhật task lên Supabase")?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Supabase PATCH error: {}", err);
        }

        self.repo.mark_tasks_synced(&[id.to_string()]).await?;
        Ok(())
    }

    /// Khởi chạy luồng chạy ngầm điều phối đồng bộ định kỳ và theo sự kiện
    pub fn start_background_worker(self: Arc<Self>, mut notify_rx: mpsc::Receiver<()>) {
        tauri::async_runtime::spawn(async move {
            // Đợi 500ms khi khởi động app rồi đồng bộ lần đầu
            sleep(Duration::from_millis(500)).await;
            let _ = self.sync_cycle().await;

            // Đồng bộ định kỳ mỗi 30 giây
            let mut periodic_interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    _ = periodic_interval.tick() => {
                        let _ = self.sync_cycle().await;
                    }
                    Some(_) = notify_rx.recv() => {
                        // Debounce 1.5s để gom nhiều thay đổi liên tiếp
                        sleep(Duration::from_millis(1500)).await;
                        while notify_rx.try_recv().is_ok() {}
                        let _ = self.sync_cycle().await;
                    }
                }
            }
        });
    }

    /// Chu kỳ đồng bộ hai chiều (Push Local -> Cloud, sau đó Pull Cloud -> Local)
    pub async fn sync_cycle(&self) -> Result<()> {
        let user = match self.repo.get_current_user().await {
            Ok(Some(u)) => u,
            _ => {
                let mut st = self.state.lock().await;
                st.is_logged_in = false;
                st.user_email = None;
                st.state = "idle".to_string();
                st.error_message = None;
                return Ok(());
            }
        };

        if !self.auth_client.is_configured() {
            let mut st = self.state.lock().await;
            st.is_logged_in = true;
            st.user_email = Some(user.email);
            st.state = "error".to_string();
            st.error_message = Some("Supabase chưa được cấu hình key trong .env".to_string());
            self.emit_sync_status(&st).await;
            return Ok(());
        }

        let access_token = match self.auth_client.get_valid_access_token(&self.repo).await {
            Ok(Some(token)) => token,
            Ok(None) => return Ok(()),
            Err(e) => {
                let mut st = self.state.lock().await;
                st.state = "error".to_string();
                st.error_message = Some(e.to_string());
                self.emit_sync_status(&st).await;
                return Ok(());
            }
        };

        // Đánh dấu trạng thái đang đồng bộ
        {
            let mut st = self.state.lock().await;
            st.is_logged_in = true;
            st.user_email = Some(user.email.clone());
            st.state = "syncing".to_string();
            st.error_message = None;
            self.emit_sync_status(&st).await;
        }

        let mut has_changes = false;

        // 1. PUSH: Đẩy dữ liệu pending từ SQLite lên Supabase PostgREST
        let push_res = self.push_pending_tasks(&user.user_id, &access_token).await;
        match push_res {
            Ok(pushed) => {
                if pushed {
                    has_changes = true;
                }
            }
            Err(ref e) => {
                let mut st = self.state.lock().await;
                st.state = "error".to_string();
                st.error_message = Some(format!("Lỗi push: {}", e));
                self.emit_sync_status(&st).await;
                return Ok(());
            }
        }

        // 2. PULL: Kéo dữ liệu mới từ Supabase về SQLite (Last-Write-Wins)
        let pull_res = self.pull_remote_tasks(&user.user_id, &access_token).await;
        match pull_res {
            Ok(pulled) => {
                if pulled {
                    has_changes = true;
                }
            }
            Err(ref e) => {
                let mut st = self.state.lock().await;
                st.state = "error".to_string();
                st.error_message = Some(format!("Lỗi pull: {}", e));
                self.emit_sync_status(&st).await;
                return Ok(());
            }
        }

        // Hoàn tất đồng bộ thành công
        let now_ms = Utc::now().timestamp_millis();
        let pending = self.repo.count_pending_tasks().await.unwrap_or(0);
        {
            let mut st = self.state.lock().await;
            st.is_logged_in = true;
            st.user_email = Some(user.email);
            st.state = "synced".to_string();
            st.last_synced_at = Some(now_ms);
            st.pending_count = pending;
            st.error_message = None;
            self.emit_sync_status(&st).await;
        }

        // Bắn sự kiện thông báo cho Vue 3 tự làm mới dữ liệu
        if has_changes {
            let handle_lock = self.app_handle.lock().await;
            if let Some(ref handle) = *handle_lock {
                let _ = handle.emit("tasks-synced", ());
            }
        }

        Ok(())
    }

    async fn emit_sync_status(&self, status: &SyncStatusState) {
        let handle_lock = self.app_handle.lock().await;
        if let Some(ref handle) = *handle_lock {
            let _ = handle.emit("sync-status-changed", status);
        }
    }

    /// Đẩy các task có sync_status = 'pending' lên Supabase (không gửi sync_status lên Supabase)
    async fn push_pending_tasks(&self, user_id: &str, access_token: &str) -> Result<bool> {
        let pending_tasks = self.repo.get_pending_sync_tasks(user_id).await?;
        if pending_tasks.is_empty() {
            return Ok(false);
        }

        let payload: Vec<serde_json::Value> = pending_tasks
            .iter()
            .map(|t| {
                json!({
                    "id": t.id,
                    "user_id": user_id,
                    "title": t.title,
                    "description": t.description,
                    "status": t.status.as_str(),
                    "priority": t.priority.as_i64(),
                    "due_date": t.due_date,
                    "start_time": t.start_time,
                    "end_time": t.end_time,
                    "created_at": t.created_at,
                    "updated_at": t.updated_at,
                    "deleted_at": t.deleted_at
                })
            })
            .collect();

        let endpoint = format!("{}/rest/v1/tasks?on_conflict=id", self.auth_client.url());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let resp = client
            .post(&endpoint)
            .header("apikey", self.auth_client.anon_key())
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Prefer", "resolution=merge-duplicates,return=minimal")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Lỗi mạng khi push task lên Supabase")?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Lỗi PostgREST push: {}", err);
        }

        // Đánh dấu các task đã được sync thành công trong SQLite
        let synced_ids: Vec<String> = pending_tasks.into_iter().map(|t| t.id).collect();
        self.repo.mark_tasks_synced(&synced_ids).await?;

        Ok(true)
    }

    /// Kéo các task cập nhật từ Supabase về SQLite theo mốc last_pulled_at
    async fn pull_remote_tasks(&self, user_id: &str, access_token: &str) -> Result<bool> {
        let meta_key = format!("last_pulled_at_{}", user_id);
        let last_pulled_at: i64 = self
            .repo
            .get_sync_meta(&meta_key)
            .await?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let endpoint = format!(
            "{}/rest/v1/tasks?updated_at=gt.{}&order=updated_at.asc",
            self.auth_client.url(),
            last_pulled_at
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let resp = client
            .get(&endpoint)
            .header("apikey", self.auth_client.anon_key())
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Lỗi mạng khi pull task từ Supabase")?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Lỗi PostgREST pull: {}", err);
        }

        let remote_tasks: Vec<Task> = resp
            .json()
            .await
            .context("Lỗi đọc danh sách task tải về từ Supabase")?;

        if remote_tasks.is_empty() {
            return Ok(false);
        }

        let mut max_updated = last_pulled_at;
        let mut has_changes = false;

        for mut task in remote_tasks {
            if task.updated_at > max_updated {
                max_updated = task.updated_at;
            }
            if task.user_id.is_none() {
                task.user_id = Some(user_id.to_string());
            }
            let updated = self.repo.upsert_pulled_task(&task).await?;
            if updated {
                has_changes = true;
            }
        }

        // Cập nhật lại mốc sync
        self.repo
            .set_sync_meta(&meta_key, &max_updated.to_string())
            .await?;

        Ok(has_changes)
    }
}

