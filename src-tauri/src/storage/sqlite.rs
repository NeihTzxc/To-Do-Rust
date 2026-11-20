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

use crate::domain::task::{CreateTaskInput, Priority, Task, TaskStatus, UpdateTaskInput};
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

    fn map_row(row: SqliteRow) -> Task {
        let status_str: String = row.get("status");
        let priority_int: i64 = row.get("priority");

        Task {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            status: TaskStatus::from_str_loose(&status_str),
            priority: Priority::from_i64(priority_int),
            due_date: row.get("due_date"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
            sync_status: row.get("sync_status"),
        }
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepo {
    async fn list_active_tasks(&self) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT id, title, description, status, priority, due_date, start_time, end_time, created_at, updated_at, deleted_at, sync_status
             FROM tasks
             WHERE deleted_at IS NULL
             ORDER BY
                 CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                 priority DESC,
                 created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Lỗi truy vấn danh sách task đang hoạt động")?;

        let tasks = rows.into_iter().map(Self::map_row).collect();
        Ok(tasks)
    }

    async fn get_task_by_id(&self, id: &str) -> Result<Option<Task>> {
        let maybe_row = sqlx::query(
            "SELECT id, title, description, status, priority, due_date, start_time, end_time, created_at, updated_at, deleted_at, sync_status
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
        let now = Utc::now().timestamp_millis();
        let id = Uuid::now_v7().to_string();
        let status = TaskStatus::Todo;
        let sync_status = "pending";

        sqlx::query(
            "INSERT INTO tasks (
                 id, title, description, status, priority,
                 due_date, start_time, end_time, created_at, updated_at, deleted_at, sync_status
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(&id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(status.as_str())
        .bind(input.priority.as_i64())
        .bind(input.due_date)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(now)
        .bind(now)
        .bind(None::<i64>)
        .bind(sync_status)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Lỗi thêm mới task '{}'", input.title))?;

        Ok(Task {
            id,
            title: input.title,
            description: input.description,
            status,
            priority: input.priority,
            due_date: input.due_date,
            start_time: input.start_time,
            end_time: input.end_time,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            sync_status: sync_status.to_string(),
        })
    }

    async fn update_task(&self, input: UpdateTaskInput) -> Result<Task> {
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
                 updated_at = $8,
                 sync_status = $9
             WHERE id = $10 AND deleted_at IS NULL",
        )
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.status.as_str())
        .bind(input.priority.as_i64())
        .bind(input.due_date)
        .bind(input.start_time)
        .bind(input.end_time)
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
        if trimmed.is_empty() {
            // Lấy 8 task mới cập nhật gần nhất khi chưa gõ từ khóa
            let rows = sqlx::query(
                "SELECT id, title, description, status, priority, due_date, start_time, end_time, created_at, updated_at, deleted_at, sync_status
                 FROM tasks
                 WHERE deleted_at IS NULL
                 ORDER BY
                     CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                     updated_at DESC
                 LIMIT 8",
            )
            .fetch_all(&self.pool)
            .await
            .context("Lỗi truy vấn task gần đây")?;

            return Ok(rows.into_iter().map(Self::map_row).collect());
        }

        let normalized_query = remove_vietnamese_diacritics(trimmed);

        // Lấy danh sách active tasks để tìm kiếm kèm chuẩn hóa tiếng Việt không dấu
        let rows = sqlx::query(
            "SELECT id, title, description, status, priority, due_date, start_time, end_time, created_at, updated_at, deleted_at, sync_status
             FROM tasks
             WHERE deleted_at IS NULL
             ORDER BY
                 CASE WHEN status = 'done' THEN 1 ELSE 0 END ASC,
                 updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Lỗi truy vấn danh sách task phục vụ tìm kiếm")?;

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
