use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use directories::BaseDirs;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow, SqliteSynchronous},
    Pool, Row, Sqlite,
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::domain::auth::{AuthSession, UserInfo};
use crate::domain::task::{
    compute_next_occurrence_ms, CreateTaskInput, Priority, Task, TaskStatus, UpdateTaskInput,
};
use crate::storage::TaskRepository;

#[derive(Clone)]
pub struct SqliteTaskRepo {
    pool: Pool<Sqlite>,
}

impl SqliteTaskRepo {
    #[allow(dead_code)]
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Khởi tạo SQLite repository tại đường dẫn mặc định ~/.local/share/rust-task-mgr/tasks.db
    /// Tự động tạo thư mục cha và chạy migration
    pub async fn new_default() -> Result<Self> {
        let db_path = Self::resolve_db_path();
        Self::new_with_path(&db_path).await
    }

    /// Khởi tạo với đường dẫn chỉ định
    pub async fn new_with_path(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Không thể tạo thư mục chứa database: {}",
                    parent.display()
                )
            })?;
        }

        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .with_context(|| format!("Không thể kết nối SQLite DB: {}", db_path.display()))?;

        // Tự động chạy migrations khi khởi động
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Lỗi khi chạy migration database SQLite")?;

        Ok(Self { pool })
    }

    /// Đường dẫn chuẩn XDG: ~/.local/share/rust-task-mgr/tasks.db
    /// Hỗ trợ biến môi trường TASK_MGR_DB_PATH để ghi đè khi cần test
    pub fn resolve_db_path() -> PathBuf {
        if let Ok(custom) = std::env::var("TASK_MGR_DB_PATH") {
            return PathBuf::from(custom);
        }

        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs
                .data_dir()
                .join("rust-task-mgr")
                .join("tasks.db")
        } else {
            PathBuf::from("tasks.db")
        }
    }

    #[allow(dead_code)]
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Lấy user_id của phiên đăng nhập hiện tại nếu có
    pub async fn current_user_id(&self) -> Result<Option<String>> {
        let maybe_row = sqlx::query("SELECT user_id FROM auth_session WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .context("Lỗi truy vấn user_id từ auth_session")?;
        Ok(maybe_row.map(|r| r.get("user_id")))
    }

    fn map_row(row: SqliteRow) -> Task {
        let status_str: String = row.get("status");
        let priority_int: i64 = row.get("priority");
        let is_all_day_int: i64 = row.try_get("is_all_day").unwrap_or(0);

        Task {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            description: row.get("description"),
            status: TaskStatus::from_str_loose(&status_str),
            priority: Priority::from_i64(priority_int),
            due_date: row.get("due_date"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            is_all_day: is_all_day_int != 0,
            recurrence_rule: row.try_get("recurrence_rule").unwrap_or(None),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
            sync_status: row.get("sync_status"),
        }
    }

    /// Nếu task chuyển từ chưa hoàn thành -> hoàn thành và có recurrence_rule,
    /// tự động sinh bản ghi kỳ tiếp theo trong SQLite (sync_status = 'pending').
    async fn spawn_next_recurring_if_needed(&self, prev: &Task, new_status: TaskStatus) -> Result<Option<Task>> {
        if prev.status.is_done() || !new_status.is_done() {
            return Ok(None);
        }
        let Some(ref rule) = prev.recurrence_rule else {
            return Ok(None);
        };
        if rule.trim().is_empty() {
            return Ok(None);
        }

        let now = Utc::now().timestamp_millis();
        let base = prev.start_time.or(prev.due_date).unwrap_or(now);
        let Some(next_base) = compute_next_occurrence_ms(base, rule) else {
            return Ok(None);
        };
        let delta = next_base - base;
        let next_due = prev.due_date.map(|d| d + delta);
        let next_start = prev.start_time.map(|s| s + delta);
        let next_end = prev.end_time.map(|e| e + delta);
        let next_id = Uuid::now_v7().to_string();

        sqlx::query(
            "INSERT INTO tasks (
                 id, user_id, title, description, status, priority,
                 due_date, start_time, end_time, is_all_day, recurrence_rule,
                 created_at, updated_at, deleted_at, sync_status
             )
             VALUES ($1, $2, $3, $4, 'todo', $5, $6, $7, $8, $9, $10, $11, $11, NULL, 'pending')",
        )
        .bind(&next_id)
        .bind(&prev.user_id)
        .bind(&prev.title)
        .bind(&prev.description)
        .bind(prev.priority.as_i64())
        .bind(next_due)
        .bind(next_start)
        .bind(next_end)
        .bind(if prev.is_all_day { 1_i64 } else { 0_i64 })
        .bind(&prev.recurrence_rule)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Lỗi tự động sinh kỳ tiếp theo cho task lặp lại")?;

        Ok(Some(Task {
            id: next_id,
            user_id: prev.user_id.clone(),
            title: prev.title.clone(),
            description: prev.description.clone(),
            status: TaskStatus::Todo,
            priority: prev.priority,
            due_date: next_due,
            start_time: next_start,
            end_time: next_end,
            is_all_day: prev.is_all_day,
            recurrence_rule: prev.recurrence_rule.clone(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            sync_status: "pending".to_string(),
        }))
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepo {
    async fn list_active_tasks(&self) -> Result<Vec<Task>> {
        let uid = self.current_user_id().await?;
        let rows = if let Some(ref user_id) = uid {
            sqlx::query(
                "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
                 FROM tasks
                 WHERE deleted_at IS NULL AND user_id = $1
                 ORDER BY
                     CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                     priority DESC,
                     created_at DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .context("Lỗi truy vấn danh sách task của user đang hoạt động")?
        } else {
            sqlx::query(
                "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
                 FROM tasks
                 WHERE deleted_at IS NULL AND user_id IS NULL
                 ORDER BY
                     CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                     priority DESC,
                     created_at DESC",
            )
            .fetch_all(&self.pool)
            .await
            .context("Lỗi truy vấn danh sách task Guest đang hoạt động")?
        };

        let tasks = rows.into_iter().map(Self::map_row).collect();
        Ok(tasks)
    }

    async fn get_task_by_id(&self, id: &str) -> Result<Option<Task>> {
        let maybe_row = sqlx::query(
            "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
             FROM tasks
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .with_context(|| format!("Lỗi truy vấn task id: {}", id))?;

        Ok(maybe_row.map(Self::map_row))
    }

    async fn create_task(&self, input: CreateTaskInput) -> Result<Task> {
        let uid = self.current_user_id().await?;
        let now = Utc::now().timestamp_millis();
        let id = Uuid::now_v7().to_string();
        let status = TaskStatus::Todo;
        let sync_status = "pending";

        sqlx::query(
            "INSERT INTO tasks (
                 id, user_id, title, description, status, priority,
                 due_date, start_time, end_time, is_all_day, recurrence_rule,
                 created_at, updated_at, deleted_at, sync_status
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        )
        .bind(&id)
        .bind(&uid)
        .bind(&input.title)
        .bind(&input.description)
        .bind(status.as_str())
        .bind(input.priority.as_i64())
        .bind(input.due_date)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(if input.is_all_day { 1_i64 } else { 0_i64 })
        .bind(&input.recurrence_rule)
        .bind(now)
        .bind(now)
        .bind(None::<i64>)
        .bind(sync_status)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Lỗi thêm mới task '{}'", input.title))?;

        Ok(Task {
            id,
            user_id: uid,
            title: input.title,
            description: input.description,
            status,
            priority: input.priority,
            due_date: input.due_date,
            start_time: input.start_time,
            end_time: input.end_time,
            is_all_day: input.is_all_day,
            recurrence_rule: input.recurrence_rule,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            sync_status: sync_status.to_string(),
        })
    }

    async fn update_task(&self, input: UpdateTaskInput) -> Result<Task> {
        let prev = self
            .get_task_by_id(&input.id)
            .await?
            .with_context(|| format!("Không tìm thấy task với ID: {} hoặc đã bị xóa", input.id))?;

        let now = Utc::now().timestamp_millis();
        let sync_status = "pending";

        let result = sqlx::query(
            "UPDATE tasks
             SET title = $1,
                 description = $2,
                 status = $3,
                 priority = $4,
                 due_date = $5,
                 start_time = $6,
                 end_time = $7,
                 is_all_day = $8,
                 recurrence_rule = $9,
                 updated_at = $10,
                 sync_status = $11
             WHERE id = $12 AND deleted_at IS NULL",
        )
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.status.as_str())
        .bind(input.priority.as_i64())
        .bind(input.due_date)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(if input.is_all_day { 1_i64 } else { 0_i64 })
        .bind(&input.recurrence_rule)
        .bind(now)
        .bind(sync_status)
        .bind(&input.id)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Lỗi cập nhật task id {}", input.id))?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Không tìm thấy task với ID: {} hoặc đã bị xóa", input.id);
        }

        let updated = self
            .get_task_by_id(&input.id)
            .await?
            .context("Không thể tải lại task sau khi cập nhật")?;

        let _ = self.spawn_next_recurring_if_needed(&prev, input.status).await?;

        Ok(updated)
    }

    async fn update_schedule(&self, id: &str, start_time: Option<i64>, end_time: Option<i64>) -> Result<Task> {
        let now = Utc::now().timestamp_millis();
        let sync_status = "pending";

        let result = sqlx::query(
            "UPDATE tasks
             SET start_time = $1,
                 end_time = $2,
                 updated_at = $3,
                 sync_status = $4
             WHERE id = $5 AND deleted_at IS NULL",
        )
        .bind(start_time)
        .bind(end_time)
        .bind(now)
        .bind(sync_status)
        .bind(id)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Lỗi cập nhật lịch task {}", id))?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Không tìm thấy task để cập nhật lịch: {}", id);
        }

        let updated = self
            .get_task_by_id(id)
            .await?
            .context("Không thể tải lại task sau khi cập nhật lịch")?;

        Ok(updated)
    }

    async fn change_status(&self, id: &str, new_status: TaskStatus) -> Result<Task> {
        let prev = self
            .get_task_by_id(id)
            .await?
            .with_context(|| format!("Không tìm thấy task để cập nhật trạng thái: {}", id))?;

        let now = Utc::now().timestamp_millis();
        let sync_status = "pending";

        let result = sqlx::query(
            "UPDATE tasks
             SET status = $1,
                 updated_at = $2,
                 sync_status = $3
             WHERE id = $4 AND deleted_at IS NULL",
        )
        .bind(new_status.as_str())
        .bind(now)
        .bind(sync_status)
        .bind(id)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Lỗi cập nhật trạng thái task {}", id))?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Không tìm thấy task để cập nhật trạng thái: {}", id);
        }

        let updated = self
            .get_task_by_id(id)
            .await?
            .context("Không thể tải lại task sau khi cập nhật trạng thái")?;

        let _ = self.spawn_next_recurring_if_needed(&prev, new_status).await?;

        Ok(updated)
    }

    async fn toggle_status(&self, id: &str) -> Result<Task> {
        let current = self
            .get_task_by_id(id)
            .await?
            .with_context(|| format!("Không tìm thấy task có id: {}", id))?;

        let new_status = current.status.toggle();
        self.change_status(id, new_status).await
    }

    async fn soft_delete_task(&self, id: &str) -> Result<()> {
        let now = Utc::now().timestamp_millis();
        let sync_status = "pending";

        let result = sqlx::query(
            "UPDATE tasks
             SET deleted_at = $1,
                 updated_at = $1,
                 sync_status = $2
             WHERE id = $3 AND deleted_at IS NULL",
        )
        .bind(now)
        .bind(sync_status)
        .bind(id)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Lỗi xóa mềm task id: {}", id))?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Không tìm thấy task để xóa: {}", id);
        }

        Ok(())
    }

    async fn search_tasks(&self, query: &str) -> Result<Vec<Task>> {
        let trimmed = query.trim();
        let uid = self.current_user_id().await?;

        if trimmed.is_empty() {
            let rows = if let Some(ref user_id) = uid {
                sqlx::query(
                    "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
                     FROM tasks
                     WHERE deleted_at IS NULL AND user_id = $1
                     ORDER BY
                         CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                         updated_at DESC
                     LIMIT 8",
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
                .context("Lỗi truy vấn task gần đây của user")?
            } else {
                sqlx::query(
                    "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
                     FROM tasks
                     WHERE deleted_at IS NULL AND user_id IS NULL
                     ORDER BY
                         CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                         updated_at DESC
                     LIMIT 8",
                )
                .fetch_all(&self.pool)
                .await
                .context("Lỗi truy vấn task gần đây Guest")?
            };

            return Ok(rows.into_iter().map(Self::map_row).collect());
        }

        let normalized_query = remove_vietnamese_diacritics(trimmed);

        let rows = if let Some(ref user_id) = uid {
            sqlx::query(
                "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
                 FROM tasks
                 WHERE deleted_at IS NULL AND user_id = $1
                 ORDER BY
                     CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                     updated_at DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .context("Lỗi truy vấn danh sách task phục vụ tìm kiếm")?
        } else {
            sqlx::query(
                "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
                 FROM tasks
                 WHERE deleted_at IS NULL AND user_id IS NULL
                 ORDER BY
                     CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                     updated_at DESC",
            )
            .fetch_all(&self.pool)
            .await
            .context("Lỗi truy vấn danh sách task Guest phục vụ tìm kiếm")?
        };

        let mut matched: Vec<Task> = rows
            .into_iter()
            .map(Self::map_row)
            .filter(|t| {
                let norm_title = remove_vietnamese_diacritics(&t.title);
                if norm_title.contains(&normalized_query) {
                    return true;
                }
                if let Some(ref desc) = t.description {
                    let norm_desc = remove_vietnamese_diacritics(desc);
                    if norm_desc.contains(&normalized_query) {
                        return true;
                    }
                }
                false
            })
            .take(50)
            .collect();

        matched.sort_by(|a, b| {
            let done_a = a.status.is_done();
            let done_b = b.status.is_done();
            match done_a.cmp(&done_b) {
                std::cmp::Ordering::Equal => b.updated_at.cmp(&a.updated_at),
                other => other,
            }
        });

        Ok(matched)
    }

    // --- Auth Session Methods ---

    async fn get_current_session(&self) -> Result<Option<AuthSession>> {
        let maybe_row = sqlx::query(
            "SELECT user_id, email, access_token, refresh_token, expires_at
             FROM auth_session
             WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await
        .context("Lỗi truy vấn auth_session")?;

        Ok(maybe_row.map(|row| AuthSession {
            user_id: row.get("user_id"),
            email: row.get("email"),
            access_token: row.get("access_token"),
            refresh_token: row.get("refresh_token"),
            expires_at: row.get("expires_at"),
        }))
    }

    async fn save_session(&self, session: &AuthSession) -> Result<()> {
        sqlx::query(
            "INSERT INTO auth_session (id, user_id, email, access_token, refresh_token, expires_at)
             VALUES (1, $1, $2, $3, $4, $5)
             ON CONFLICT(id) DO UPDATE SET
                 user_id = $1,
                 email = $2,
                 access_token = $3,
                 refresh_token = $4,
                 expires_at = $5",
        )
        .bind(&session.user_id)
        .bind(&session.email)
        .bind(&session.access_token)
        .bind(&session.refresh_token)
        .bind(session.expires_at)
        .execute(&self.pool)
        .await
        .context("Lỗi lưu auth_session")?;

        // Tự động gán các task Guest (user_id IS NULL) sang user_id mới đăng nhập
        self.migrate_guest_tasks_to_user(&session.user_id).await?;

        Ok(())
    }

    async fn clear_session(&self) -> Result<()> {
        sqlx::query("DELETE FROM auth_session WHERE id = 1")
            .execute(&self.pool)
            .await
            .context("Lỗi xóa auth_session khi đăng xuất")?;
        Ok(())
    }

    async fn get_current_user(&self) -> Result<Option<UserInfo>> {
        let maybe_session = self.get_current_session().await?;
        Ok(maybe_session.map(|s| UserInfo {
            user_id: s.user_id,
            email: s.email,
        }))
    }

    // --- Sync Support Methods ---

    async fn get_sync_meta(&self, key: &str) -> Result<Option<String>> {
        let maybe_row = sqlx::query("SELECT value FROM sync_meta WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .context("Lỗi truy vấn sync_meta")?;
        Ok(maybe_row.map(|r| r.get("value")))
    }

    async fn set_sync_meta(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO sync_meta (key, value)
             VALUES ($1, $2)
             ON CONFLICT(key) DO UPDATE SET value = $2",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Lỗi lưu sync_meta")?;
        Ok(())
    }

    async fn get_pending_sync_tasks(&self, user_id: &str) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT id, user_id, title, description, status, priority, due_date, start_time, end_time, is_all_day, recurrence_rule, created_at, updated_at, deleted_at, sync_status
             FROM tasks
             WHERE user_id = $1 AND sync_status = 'pending'
             ORDER BY updated_at ASC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Lỗi lấy danh sách task pending để sync")?;

        Ok(rows.into_iter().map(Self::map_row).collect())
    }

    async fn mark_tasks_synced(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        for chunk in ids.chunks(50) {
            let mut builder: sqlx::QueryBuilder<sqlx::Sqlite> =
                sqlx::QueryBuilder::new("UPDATE tasks SET sync_status = 'synced' WHERE id IN (");
            let mut separated = builder.separated(", ");
            for id in chunk {
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
            builder
                .build()
                .execute(&self.pool)
                .await
                .context("Lỗi cập nhật sync_status = 'synced'")?;
        }
        Ok(())
    }

    async fn upsert_pulled_task(&self, remote: &Task) -> Result<bool> {
        let existing = sqlx::query(
            "SELECT id, updated_at, sync_status FROM tasks WHERE id = $1"
        )
        .bind(&remote.id)
        .fetch_optional(&self.pool)
        .await
        .context("Lỗi kiểm tra task tồn tại khi pull")?;

        match existing {
            None => {
                // Task chưa có ở local -> Chèn mới
                sqlx::query(
                    "INSERT INTO tasks (
                         id, user_id, title, description, status, priority,
                         due_date, start_time, end_time, is_all_day, recurrence_rule,
                         created_at, updated_at, deleted_at, sync_status
                     )
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, 'synced')",
                )
                .bind(&remote.id)
                .bind(&remote.user_id)
                .bind(&remote.title)
                .bind(&remote.description)
                .bind(remote.status.as_str())
                .bind(remote.priority.as_i64())
                .bind(remote.due_date)
                .bind(remote.start_time)
                .bind(remote.end_time)
                .bind(if remote.is_all_day { 1_i64 } else { 0_i64 })
                .bind(&remote.recurrence_rule)
                .bind(remote.created_at)
                .bind(remote.updated_at)
                .bind(remote.deleted_at)
                .execute(&self.pool)
                .await
                .context("Lỗi chèn task pulled từ cloud")?;

                Ok(true)
            }
            Some(row) => {
                let local_updated: i64 = row.get("updated_at");
                let local_sync: String = row.get("sync_status");

                // Last-Write-Wins: Chỉ ghi đè nếu local đã synced và remote mới hơn
                if local_sync == "synced" && remote.updated_at > local_updated {
                    sqlx::query(
                        "UPDATE tasks
                         SET user_id = $1,
                             title = $2,
                             description = $3,
                             status = $4,
                             priority = $5,
                             due_date = $6,
                             start_time = $7,
                             end_time = $8,
                             is_all_day = $9,
                             recurrence_rule = $10,
                             updated_at = $11,
                             deleted_at = $12,
                             sync_status = 'synced'
                         WHERE id = $13",
                    )
                    .bind(&remote.user_id)
                    .bind(&remote.title)
                    .bind(&remote.description)
                    .bind(remote.status.as_str())
                    .bind(remote.priority.as_i64())
                    .bind(remote.due_date)
                    .bind(remote.start_time)
                    .bind(remote.end_time)
                    .bind(if remote.is_all_day { 1_i64 } else { 0_i64 })
                    .bind(&remote.recurrence_rule)
                    .bind(remote.updated_at)
                    .bind(remote.deleted_at)
                    .bind(&remote.id)
                    .execute(&self.pool)
                    .await
                    .context("Lỗi cập nhật task pulled từ cloud (LWW)")?;

                    Ok(true)
                } else {
                    // Local đang có thay đổi chưa sync hoặc mới hơn -> giữ nguyên
                    Ok(false)
                }
            }
        }
    }

    async fn migrate_guest_tasks_to_user(&self, user_id: &str) -> Result<u64> {
        let now = Utc::now().timestamp_millis();
        let result = sqlx::query(
            "UPDATE tasks
             SET user_id = $1,
                 sync_status = 'pending',
                 updated_at = $2
             WHERE user_id IS NULL",
        )
        .bind(user_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .context("Lỗi chuyển đổi guest tasks sang user account")?;

        Ok(result.rows_affected())
    }

    async fn count_pending_tasks(&self) -> Result<i64> {
        let uid = self.current_user_id().await?;
        let count_row = if let Some(ref user_id) = uid {
            sqlx::query("SELECT COUNT(*) as count FROM tasks WHERE user_id = $1 AND sync_status = 'pending'")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT COUNT(*) as count FROM tasks WHERE user_id IS NULL AND sync_status = 'pending'")
                .fetch_one(&self.pool)
                .await?
        };

        let count: i64 = count_row.get("count");
        Ok(count)
    }
}

/// Chuẩn hóa chuỗi tiếng Việt về dạng chữ thường không dấu phục vụ tìm kiếm
pub fn remove_vietnamese_diacritics(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ'
            | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ'
            | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'a',
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'e',
            'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'e',
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'i',
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ'
            | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'o',
            'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ'
            | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'o',
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'u',
            'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'u',
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
            'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'y',
            'đ' | 'Đ' => 'd',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}
