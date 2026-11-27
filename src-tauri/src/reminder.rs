use chrono::{Local, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri_plugin_notification::NotificationExt;
use tokio::time::{interval, Duration};

use crate::domain::task::Task;
use crate::storage::TaskRepository;

/// Wrapper type cho trạng thái bật/tắt thông báo cục bộ trong Tauri State
#[derive(Clone)]
pub struct NotificationsEnabled(pub Arc<AtomicBool>);

/// Cửa sổ trễ tối đa (5 phút) để vẫn gửi thông báo nếu app vừa mở sau giờ hẹn một chút,
/// nhưng tránh spam các task quá hạn từ hôm qua.
const NOTIFY_GRACE_WINDOW_MS: i64 = 5 * 60 * 1000;

/// Xác định thời điểm bắt đầu (epoch ms) cần nhắc nhở của một task.
/// Chỉ áp dụng cho task chưa hoàn thành, không phải Cả ngày (All-day), và có `start_time`.
pub fn effective_task_start_ms(task: &Task) -> Option<i64> {
    if task.status.is_done() || task.is_all_day {
        return None;
    }
    let start_ms = task.start_time?;

    let start_dt = Local.timestamp_millis_opt(start_ms).single()?;

    // Nếu là task kéo dài nhiều ngày (start_time và end_time khác ngày), bỏ qua nhắc theo giờ
    if let Some(end_ms) = task.end_time {
        if let Some(end_dt) = Local.timestamp_millis_opt(end_ms).single() {
            if start_dt.date_naive() != end_dt.date_naive() {
                return None;
            }
        }
    }

    // Nếu due_date nằm ở ngày khác với start_time, chiếu giờ của start_time lên ngày due_date
    if let Some(due_ms) = task.due_date {
        if let Some(due_dt) = Local.timestamp_millis_opt(due_ms).single() {
            if due_dt.date_naive() != start_dt.date_naive() {
                let projected = due_dt
                    .date_naive()
                    .and_time(start_dt.time())
                    .and_local_timezone(Local)
                    .single()?;
                return Some(projected.timestamp_millis());
            }
        }
    }

    Some(start_ms)
}

/// Kiểm tra task có đến giờ cần gửi thông báo tại `now_ms` hay không.
/// Trả về `Some(start_ms)` nếu cần gửi thông báo và chưa từng gửi cho mốc `start_ms` này.
pub fn should_notify_task(
    task: &Task,
    now_ms: i64,
    notified: &HashMap<String, i64>,
) -> Option<i64> {
    let start_ms = effective_task_start_ms(task)?;

    if start_ms <= now_ms && (now_ms - start_ms) <= NOTIFY_GRACE_WINDOW_MS {
        if notified.get(&task.id) != Some(&start_ms) {
            return Some(start_ms);
        }
    }

    None
}

/// Khởi chạy luồng nền kiểm tra lịch task mỗi 10 giây và gửi thông báo Desktop khi tới giờ
pub fn start_reminder_worker(
    app_handle: tauri::AppHandle,
    repo: Arc<dyn TaskRepository>,
    enabled: Arc<AtomicBool>,
) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        let mut notified: HashMap<String, i64> = HashMap::new();

        loop {
            ticker.tick().await;

            if !enabled.load(Ordering::SeqCst) {
                continue;
            }

            let now_ms = Utc::now().timestamp_millis();

            if let Ok(tasks) = repo.list_active_tasks().await {
                for task in tasks {
                    if let Some(start_ms) = should_notify_task(&task, now_ms, &notified) {
                        let time_str = Local
                            .timestamp_millis_opt(start_ms)
                            .single()
                            .map(|dt| dt.format("%H:%M").to_string())
                            .unwrap_or_else(|| "Bây giờ".to_string());

                        let body = format!("{} (Lúc {})", task.title, time_str);

                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("⏰ Đến giờ làm việc!")
                            .body(&body)
                            .show();

                        notified.insert(task.id.clone(), start_ms);
                    }
                }
            }

            // Dọn các bản ghi đã thông báo quá 24 giờ để tiết kiệm bộ nhớ
            notified.retain(|_, &mut ts| now_ms - ts < 24 * 3600 * 1000);
        }
    });
}
