pub mod sqlite;

use anyhow::Result;
use async_trait::async_trait;
use crate::domain::auth::{AuthSession, UserInfo};
use crate::domain::task::{CreateTaskInput, Task, TaskStatus, UpdateTaskInput};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Lấy danh sách các task chưa bị xóa mềm (deleted_at IS NULL) của user hiện tại (hoặc guest nếu chưa đăng nhập)
    async fn list_active_tasks(&self) -> Result<Vec<Task>>;

    /// Lấy chi tiết một task theo ID
    async fn get_task_by_id(&self, id: &str) -> Result<Option<Task>>;

    /// Tạo một task mới (tự động gắn user_id của session hiện tại)
    async fn create_task(&self, input: CreateTaskInput) -> Result<Task>;

    /// Cập nhật thông tin task
    async fn update_task(&self, input: UpdateTaskInput) -> Result<Task>;

    /// Cập nhật thời gian xếp lịch (start_time, end_time) cho Calendar time-blocking
    async fn update_schedule(&self, id: &str, start_time: Option<i64>, end_time: Option<i64>) -> Result<Task>;

    /// Đổi trạng thái task (todo/in_progress -> done hoặc done -> todo)
    async fn toggle_status(&self, id: &str) -> Result<Task>;

    /// Cập nhật trực tiếp trạng thái task (Todo, InProgress, Done)
    async fn change_status(&self, id: &str, status: TaskStatus) -> Result<Task>;

    /// Xóa mềm task (gán deleted_at = now, sync_status = 'pending')
    async fn soft_delete_task(&self, id: &str) -> Result<()>;

    /// Tìm kiếm task theo từ khóa (hỗ trợ tiếng Việt không dấu, tìm trong title & description)
    async fn search_tasks(&self, query: &str) -> Result<Vec<Task>>;

    // --- Quản lý phiên đăng nhập (Auth Session) ---
    async fn get_current_session(&self) -> Result<Option<AuthSession>>;
    async fn save_session(&self, session: &AuthSession) -> Result<()>;
    async fn clear_session(&self) -> Result<()>;
    async fn get_current_user(&self) -> Result<Option<UserInfo>>;

    // --- Quản lý đồng bộ (Sync Engine Support) ---
    async fn get_sync_meta(&self, key: &str) -> Result<Option<String>>;
    async fn set_sync_meta(&self, key: &str, value: &str) -> Result<()>;
    async fn get_pending_sync_tasks(&self, user_id: &str) -> Result<Vec<Task>>;
    async fn mark_tasks_synced(&self, ids: &[String]) -> Result<()>;
    async fn upsert_pulled_task(&self, task: &Task) -> Result<bool>;
    async fn migrate_guest_tasks_to_user(&self, user_id: &str) -> Result<u64>;
    async fn count_pending_tasks(&self) -> Result<i64>;
}

pub use sqlite::SqliteTaskRepo;
