pub mod commands;
pub mod domain;
pub mod storage;
pub mod sync;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::storage::{SqliteTaskRepo, TaskRepository};
use crate::sync::{SupabaseAuthClient, SyncEngine};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Nạp biến môi trường từ .env ở thư mục gốc hoặc thư mục src-tauri
    let _ = dotenvy::from_filename("src-tauri/.env")
        .or_else(|_| dotenvy::from_filename(".env"))
        .or_else(|_| dotenvy::dotenv());

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
    let close_to_tray = Arc::new(AtomicBool::new(true));
    let close_to_tray_window = close_to_tray.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(move |app| {
            let handle = app.handle().clone();
            let engine = sync_engine_setup.clone();
            tauri::async_runtime::block_on(async move {
                engine.set_app_handle(handle).await;
            });

            // Bắt đầu background worker cho sync hai chiều bên trong runtime của Tauri
            sync_engine_setup.start_background_worker(notify_rx);

            // Xây dựng System Tray menu
            let show_i = MenuItem::with_id(app, "show", "Mở TickTask", true, None::<&str>)?;
            let sync_i = MenuItem::with_id(app, "sync_now", "Đồng bộ ngay", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Thoát hoàn toàn", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &sync_i, &sep, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("TickTask - Task Manager")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "sync_now" => {
                            let engine = app.state::<Arc<SyncEngine>>();
                            engine.trigger_sync();
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(move |window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    if close_to_tray_window.load(Ordering::SeqCst) {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .manage(repo)
        .manage(auth_client)
        .manage(sync_engine)
        .manage(close_to_tray)
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
            commands::get_sync_status,
            commands::exit_app,
            commands::set_close_to_tray,
            commands::get_close_to_tray
        ])
        .run(tauri::generate_context!())
        .expect("Lỗi khi chạy ứng dụng Tauri");
}
