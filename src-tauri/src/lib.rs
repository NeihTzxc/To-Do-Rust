pub mod commands;
pub mod domain;
pub mod storage;

use std::sync::Arc;
use crate::storage::{SqliteTaskRepo, TaskRepository};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let repo = tauri::async_runtime::block_on(async {
        SqliteTaskRepo::new_default()
            .await
            .expect("Không thể khởi tạo cơ sở dữ liệu SQLite")
    });

    let repo: Arc<dyn TaskRepository> = Arc::new(repo);

    tauri::Builder::default()
        .manage(repo)
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::create_task,
            commands::update_task,
            commands::update_task_schedule,
            commands::toggle_task_status,
            commands::move_task_status,
            commands::delete_task,
            commands::search_tasks
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy ứng dụng Tauri");
}
