pub mod sqlite;

use anyhow::Result;
use async_trait::async_trait;
use crate::domain::task::{CreateTaskInput, Task, TaskStatus, UpdateTaskInput};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Lấy danh sách các task chưa bị xóa mềm (deleted_at IS NULL)
    async fn list_active_tasks(&self) -> Result<Vec<Task>>;

    /// Lấy chi tiết một task theo ID
    async fn get_task_by_id(&self, id: &str) -> Result<Option<Task>>;

    /// Tạo một task mới
    async fn create_task(&self, input: CreateTaskInput) -> Result<Task>;

    /// Cập nhật thông tin task
    async fn update_task(&self, input: UpdateTaskInput) -> Result<Task>;

    /// Đổi trạng thái task (todo/in_progress -> done hoặc done -> todo)
    async fn toggle_status(&self, id: &str) -> Result<Task>;

    /// Cập nhật trực tiếp trạng thái task (Todo, InProgress, Done)
    async fn change_status(&self, id: &str, status: TaskStatus) -> Result<Task>;

    /// Xóa mềm task (gán deleted_at = now, sync_status = 'pending')
    async fn soft_delete_task(&self, id: &str) -> Result<()>;
}

pub use sqlite::SqliteTaskRepo;
