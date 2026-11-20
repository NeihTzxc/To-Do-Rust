pub mod commands;
pub mod domain;
pub mod storage;
pub mod sync;

use std::sync::Arc;
use crate::storage::{SqliteTaskRepo, TaskRepository};
use crate::sync::{SupabaseAuthClient, SyncEngine};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Nạp biến môi trường từ .env nếu có
    dotenvy::dotenv().ok();

    let repo = tauri::async_runtime::block_on(async {
        SqliteTaskRepo::new_default()
            .await
            .expect("Không thể khởi tạo cơ sở dữ liệu SQLite")
    });

    let repo: Arc<dyn TaskRepository> = Arc::new(repo);
    let auth_client = Arc::new(SupabaseAuthClient::new());

    let (sync_engine, notify_rx) = SyncEngine::new(repo.clone(), (*auth_client).clone());
    let sync_engine = Arc::new(sync_engine);

    let sync_engine_setup = sync_engine.clone();

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let engine = sync_engine_setup.clone();
            tauri::async_runtime::spawn(async move {
                engine.set_app_handle(handle).await;
            });

            // Bắt đầu background worker cho sync hai chiều bên trong runtime của Tauri
            sync_engine_setup.start_background_worker(notify_rx);

            Ok(())
        })
        .manage(repo)
        .manage(auth_client)
        .manage(sync_engine)
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::create_task,
            commands::update_task,
            commands::update_task_schedule,
            commands::toggle_task_status,
            commands::move_task_status,
            commands::delete_task,
            commands::search_tasks,
            commands::login,
            commands::register,
            commands::logout,
            commands::get_current_user,
            commands::trigger_sync_now,
            commands::get_sync_status
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy ứng dụng Tauri");
}
