use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};

use crate::domain::auth::SyncStatusState;
use crate::domain::task::Task;
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

    /// Khởi chạy luồng chạy ngầm điều phối đồng bộ hai chiều
    pub fn start_background_worker(self: Arc<Self>, mut notify_rx: mpsc::Receiver<()>) {
        tauri::async_runtime::spawn(async move {
            // Đợi 2 giây khi khởi động app rồi đồng bộ lần đầu
            sleep(Duration::from_secs(2)).await;
            let _ = self.sync_cycle().await;

            let mut periodic_interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                tokio::select! {
                    _ = periodic_interval.tick() => {
                        let _ = self.sync_cycle().await;
                    }
                    Some(_) = notify_rx.recv() => {
                        // Debounce 1.5s để gom nhiều thay đổi liên tiếp (kéo-thả, gõ text)
                        sleep(Duration::from_millis(1500)).await;
                        // Xả sạch các event dồn trong channel
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
            st.state = "offline".to_string();
            st.error_message = Some("Supabase chưa được cấu hình key trong .env".to_string());
            return Ok(());
        }

        let access_token = match self.auth_client.get_valid_access_token(&self.repo).await {
            Ok(Some(token)) => token,
            Ok(None) => return Ok(()),
            Err(e) => {
                let mut st = self.state.lock().await;
                st.state = "offline".to_string();
                st.error_message = Some(e.to_string());
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
        }

        // 1. PUSH: Đẩy dữ liệu pending từ SQLite lên Supabase PostgREST
        let push_res = self.push_pending_tasks(&user.user_id, &access_token).await;
        if let Err(ref e) = push_res {
            let mut st = self.state.lock().await;
            st.state = "offline".to_string();
            st.error_message = Some(format!("Lỗi push: {}", e));
            return Ok(());
        }

        // 2. PULL: Kéo dữ liệu mới từ Supabase về SQLite (Last-Write-Wins)
        let pull_res = self.pull_remote_tasks(&user.user_id, &access_token).await;
        if let Err(ref e) = pull_res {
            let mut st = self.state.lock().await;
            st.state = "offline".to_string();
            st.error_message = Some(format!("Lỗi pull: {}", e));
            return Ok(());
        }

        // Hoàn tất đồng bộ thành công
        let now_ms = Utc::now().timestamp_millis();
        let pending = self.repo.count_pending_tasks().await.unwrap_or(0);
        {
            let mut st = self.state.lock().await;
            st.is_logged_in = true;
            st.user_email = Some(user.email);
            st.state = "idle".to_string();
            st.last_synced_at = Some(now_ms);
            st.pending_count = pending;
            st.error_message = None;
        }

        Ok(())
    }

    /// Đẩy các task có sync_status = 'pending' lên Supabase
    async fn push_pending_tasks(&self, user_id: &str, access_token: &str) -> Result<()> {
        let pending_tasks = self.repo.get_pending_sync_tasks(user_id).await?;
        if pending_tasks.is_empty() {
            return Ok(());
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

        Ok(())
    }

    /// Kéo các task cập nhật từ Supabase về SQLite theo mốc last_pulled_at
    async fn pull_remote_tasks(&self, user_id: &str, access_token: &str) -> Result<()> {
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
            return Ok(());
        }

        let mut max_updated = last_pulled_at;
        let mut has_changes = false;

        for task in remote_tasks {
            if task.updated_at > max_updated {
                max_updated = task.updated_at;
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

        // Nếu có thay đổi mới từ cloud, bắn sự kiện thông báo cho Vue 3 tự làm mới dữ liệu
        if has_changes {
            let handle_lock = self.app_handle.lock().await;
            if let Some(ref handle) = *handle_lock {
                let _ = handle.emit("tasks-synced", ());
            }
        }

        Ok(())
    }
}

