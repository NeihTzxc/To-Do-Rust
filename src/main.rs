mod domain;
mod storage;
mod ui;

use eframe::egui;
use std::sync::mpsc;
use tokio::sync::mpsc as tokio_mpsc;

use crate::storage::{SqliteTaskRepo, TaskRepository};
use crate::ui::app::{setup_vietnamese_fonts, DbCommand, DbEvent};
use crate::ui::TaskApp;

fn main() -> Result<(), eframe::Error> {
    // 1. Khởi tạo Tokio Runtime (đa luồng)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Không thể khởi tạo Tokio runtime");

    // 2. Chạy migration SQLite và khởi tạo database
    let db_path = SqliteTaskRepo::resolve_db_path();
    let db_path_str = db_path.display().to_string();

    let repo = rt
        .block_on(async { SqliteTaskRepo::new_default().await })
        .unwrap_or_else(|err| {
            eprintln!("Lỗi khởi tạo SQLite Database: {:?}", err);
            panic!("Không thể khởi tạo cơ sở dữ liệu: {}", err);
        });

    // 3. Khởi tạo channel giao tiếp giữa UI thread và Tokio background worker
    let (tx_cmd, mut rx_cmd) = tokio_mpsc::unbounded_channel::<DbCommand>();
    let (tx_event, rx_event) = mpsc::channel::<DbEvent>();

    // 4. Spawn background worker trong Tokio runtime để xử lý các truy vấn DB bất đồng bộ
    let repo_worker = repo.clone();
    let tx_event_worker = tx_event.clone();

    rt.spawn(async move {
        // Tải danh sách công việc ban đầu
        match repo_worker.list_active_tasks().await {
            Ok(tasks) => {
                let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
            }
            Err(e) => {
                let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi nạp task: {}", e)));
            }
        }

        // Lắng nghe các lệnh từ giao diện UI
        while let Some(cmd) = rx_cmd.recv().await {
            match cmd {
                DbCommand::Refresh => {
                    match repo_worker.list_active_tasks().await {
                        Ok(tasks) => {
                            let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
                        }
                        Err(e) => {
                            let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi làm mới: {}", e)));
                        }
                    }
                }
                DbCommand::CreateTask {
                    title,
                    description,
                    priority,
                } => {
                    let input = crate::domain::CreateTaskInput {
                        title,
                        description,
                        priority,
                        due_date: None,
                    };
                    match repo_worker.create_task(input).await {
                        Ok(created) => {
                            let _ = tx_event_worker.send(DbEvent::Notification(format!(
                                "Đã thêm công việc: {}",
                                created.title
                            )));
                            if let Ok(tasks) = repo_worker.list_active_tasks().await {
                                let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
                            }
                        }
                        Err(e) => {
                            let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi thêm việc: {}", e)));
                        }
                    }
                }
                DbCommand::UpdateTask(input) => {
                    match repo_worker.update_task(input).await {
                        Ok(updated) => {
                            let _ = tx_event_worker.send(DbEvent::Notification(format!(
                                "Đã cập nhật công việc: {}",
                                updated.title
                            )));
                            if let Ok(tasks) = repo_worker.list_active_tasks().await {
                                let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
                            }
                        }
                        Err(e) => {
                            let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi cập nhật: {}", e)));
                        }
                    }
                }
                DbCommand::ToggleStatus(id) => {
                    match repo_worker.toggle_status(&id).await {
                        Ok(task) => {
                            let status_lbl = task.status.label();
                            let _ = tx_event_worker.send(DbEvent::Notification(format!(
                                "Đã chuyển '{}' sang '{}'",
                                task.title, status_lbl
                            )));
                            if let Ok(tasks) = repo_worker.list_active_tasks().await {
                                let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
                            }
                        }
                        Err(e) => {
                            let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi đổi trạng thái: {}", e)));
                        }
                    }
                }
                DbCommand::ChangeStatus { id, status } => {
                    match repo_worker.change_status(&id, status).await {
                        Ok(task) => {
                            let status_lbl = task.status.label();
                            let _ = tx_event_worker.send(DbEvent::Notification(format!(
                                "Đã chuyển '{}' sang '{}'",
                                task.title, status_lbl
                            )));
                            if let Ok(tasks) = repo_worker.list_active_tasks().await {
                                let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
                            }
                        }
                        Err(e) => {
                            let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi đổi trạng thái: {}", e)));
                        }
                    }
                }
                DbCommand::SoftDelete(id) => {
                    match repo_worker.soft_delete_task(&id).await {
                        Ok(()) => {
                            let _ = tx_event_worker.send(DbEvent::Notification(
                                "Đã xóa mềm công việc (sync_status = 'pending')".to_string(),
                            ));
                            if let Ok(tasks) = repo_worker.list_active_tasks().await {
                                let _ = tx_event_worker.send(DbEvent::LoadedTasks(tasks));
                            }
                        }
                        Err(e) => {
                            let _ = tx_event_worker.send(DbEvent::Error(format!("Lỗi xóa công việc: {}", e)));
                        }
                    }
                }
            }
        }
    });

    // 5. Cấu hình native window options cho eframe
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Personal Task Manager - Local First")
            .with_inner_size([1050.0, 720.0])
            .with_min_inner_size([600.0, 480.0]),
        ..Default::default()
    };

    // 6. Khởi chạy ứng dụng Desktop UI (eframe event loop)
    eframe::run_native(
        "Personal Task Manager",
        native_options,
        Box::new(move |cc| {
            // Nạp font hệ thống tiếng Việt
            setup_vietnamese_fonts(&cc.egui_ctx);

            Ok(Box::new(TaskApp::new(tx_cmd, rx_event, db_path_str)))
        }),
    )
}
