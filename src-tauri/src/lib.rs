pub mod commands;
pub mod domain;
pub mod storage;
pub mod sync;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::storage::{SqliteTaskRepo, TaskRepository};
use crate::sync::{SupabaseAuthClient, SyncEngine};

const TRAY_ICON_ID: &str = "main-tray";

fn today_task_count(tasks: &[crate::domain::task::Task]) -> usize {
    let now = chrono::Local::now();
    let start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("valid start of day")
        .and_local_timezone(now.timezone())
        .single()
        .expect("valid local start of day")
        .timestamp_millis();
    let end = start + 24 * 60 * 60 * 1000 - 1;

    tasks
        .iter()
        .filter(|task| {
            task.status != crate::domain::task::TaskStatus::Done
                && task.due_date.is_some_and(|due| due >= start && due <= end)
        })
        .count()
}

fn draw_badge(icon: &mut image::RgbaImage, count: usize) {
    if count == 0 {
        return;
    }

    let center_x = icon.width() as i32 - 9;
    let center_y = icon.height() as i32 - 9;
    let radius = 8;
    let badge = image::Rgba([255, 255, 255, 255]);

    for y in (center_y - radius)..=(center_y + radius) {
        for x in (center_x - radius)..=(center_x + radius) {
            if (x - center_x).pow(2) + (y - center_y).pow(2) <= radius.pow(2)
                && x >= 0
                && y >= 0
                && x < icon.width() as i32
                && y < icon.height() as i32
            {
                icon.put_pixel(x as u32, y as u32, badge);
            }
        }
    }

    // Tiny 3x5 bitmap font so the badge does not need a bundled font file.
    let label = if count > 99 { "99+".to_string() } else { count.to_string() };
    let patterns: std::collections::HashMap<char, [u8; 5]> = [
        ('0', [0b111, 0b101, 0b101, 0b101, 0b111]),
        ('1', [0b010, 0b110, 0b010, 0b010, 0b111]),
        ('2', [0b110, 0b001, 0b010, 0b100, 0b111]),
        ('3', [0b110, 0b001, 0b010, 0b001, 0b110]),
        ('4', [0b101, 0b101, 0b111, 0b001, 0b001]),
        ('5', [0b111, 0b100, 0b110, 0b001, 0b110]),
        ('6', [0b011, 0b100, 0b111, 0b101, 0b111]),
        ('7', [0b111, 0b001, 0b010, 0b010, 0b010]),
        ('8', [0b111, 0b101, 0b111, 0b101, 0b111]),
        ('9', [0b111, 0b101, 0b111, 0b001, 0b110]),
        ('+', [0b000, 0b010, 0b111, 0b010, 0b000]),
    ]
    .into_iter()
    .collect();

    let scale = 1i32;
    let char_width = 4i32;
    let text_width = label.chars().count() as i32 * char_width - 1;
    let start_x = center_x - text_width / 2;
    let start_y = center_y - 2;
    for (char_index, character) in label.chars().enumerate() {
        if let Some(pattern) = patterns.get(&character) {
            let offset_x = start_x + char_index as i32 * char_width;
            for (row, bits) in pattern.iter().enumerate() {
                for col in 0..3 {
                    if bits & (1 << (2 - col)) != 0 {
                        let x = offset_x + col * scale;
                        let y = start_y + row as i32 * scale;
                        if x >= 0 && y >= 0 && x < icon.width() as i32 && y < icon.height() as i32 {
                            // Transparent digits stay legible when macOS recolors the icon.
                            icon.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 0]));
                        }
                    }
                }
            }
        }
    }
}

fn tray_icon(count: usize) -> tauri::Result<Image<'static>> {
    let mut icon = image::RgbaImage::new(32, 32);
    let white = image::Rgba([255, 255, 255, 255]);

    // Draw a monochrome check with a transparent background at tray size.
    for y in 1..31 {
        for x in 1..31 {
            let outer = inside_rounded_rect(x, y, 1, 30, 7);
            let inner = inside_rounded_rect(x, y, 5, 26, 5);
            if outer && !inner {
                icon.put_pixel(x, y, white);
            }
        }
    }

    draw_white_line(&mut icon, (8.0, 16.0), (13.0, 21.0), 1.9);
    draw_white_line(&mut icon, (13.0, 21.0), (24.0, 10.0), 1.9);

    draw_badge(&mut icon, count);
    Ok(Image::new_owned(icon.into_raw(), 32, 32))
}

fn inside_rounded_rect(x: u32, y: u32, min: u32, max: u32, radius: u32) -> bool {
    if x < min || x > max || y < min || y > max {
        return false;
    }
    let nearest_x = x.clamp(min + radius, max - radius);
    let nearest_y = y.clamp(min + radius, max - radius);
    (x as i32 - nearest_x as i32).pow(2) + (y as i32 - nearest_y as i32).pow(2)
        <= (radius as i32).pow(2)
}

fn draw_white_line(icon: &mut image::RgbaImage, start: (f32, f32), end: (f32, f32), radius: f32) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length_squared = dx * dx + dy * dy;
    for y in 0..icon.height() {
        for x in 0..icon.width() {
            let px = x as f32 - start.0;
            let py = y as f32 - start.1;
            let t = ((px * dx + py * dy) / length_squared).clamp(0.0, 1.0);
            let distance_squared = (px - t * dx).powi(2) + (py - t * dy).powi(2);
            if distance_squared <= radius * radius {
                icon.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
            }
        }
    }
}

fn start_tray_badge_worker(app: tauri::AppHandle, repo: Arc<dyn TaskRepository>) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Ok(tasks) = repo.list_active_tasks().await {
                if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
                    let count = today_task_count(&tasks);
                    if let Ok(icon) = tray_icon(count) {
                        let _ = tray.set_icon(Some(icon));
                    }
                    let _ = tray.set_tooltip(Some(format!(
                        "TickTask - {} task cần làm hôm nay",
                        count
                    )));
                }
            }
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });
}

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
    let tray_repo = repo.clone();

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

            let _tray = TrayIconBuilder::with_id(TRAY_ICON_ID)
                .icon(tray_icon(0)?)
                .icon_as_template(true)
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

            start_tray_badge_worker(app.handle().clone(), tray_repo.clone());

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
