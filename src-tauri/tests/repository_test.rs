use rust_task_mgr_lib::domain::task::{CreateTaskInput, Priority, TaskStatus, UpdateTaskInput};
use rust_task_mgr_lib::storage::{SqliteTaskRepo, TaskRepository};

#[tokio::test]
async fn test_sqlite_repository_crud_and_sync() {
    // Sử dụng SQLite file tạm thời trong thư mục temp
    let temp_dir = tempfile::tempdir().expect("Không thể tạo temp dir");
    let db_path = temp_dir.path().join("test_tasks.db");

    let repo = SqliteTaskRepo::new_with_path(&db_path)
        .await
        .expect("Khởi tạo SqliteTaskRepo thất bại");

    // 1. Tạo task mới
    let input = CreateTaskInput {
        title: "Học Rust và Egui".to_string(),
        description: Some("Xây dựng ứng dụng Desktop Local-first".to_string()),
        priority: Priority::High,
        due_date: None,
        start_time: Some(1735689600000),
        end_time: Some(1735693200000),
        is_all_day: false,
        recurrence_rule: None,
    };

    let created = repo.create_task(input).await.expect("Tạo task thất bại");
    assert_eq!(created.title, "Học Rust và Egui");
    assert_eq!(created.status, TaskStatus::Todo);
    assert_eq!(created.priority, Priority::High);
    assert_eq!(created.sync_status, "pending");
    assert!(created.deleted_at.is_none());
    assert!(!created.id.is_empty());

    // 2. Danh sách active tasks
    let tasks = repo.list_active_tasks().await.expect("Lấy danh sách task thất bại");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, created.id);

    // 3. Đổi trạng thái (toggle_status)
    let toggled = repo.toggle_status(&created.id).await.expect("Toggle task thất bại");
    assert_eq!(toggled.status, TaskStatus::Done);
    assert_eq!(toggled.sync_status, "pending");
    assert!(toggled.updated_at >= created.updated_at);

    // Toggle ngược lại
    let toggled_back = repo.toggle_status(&created.id).await.expect("Toggle task lần 2 thất bại");
    assert_eq!(toggled_back.status, TaskStatus::Todo);

    // Test change_status trực tiếp (phục vụ Kanban Drag & Drop)
    let changed = repo.change_status(&created.id, TaskStatus::InProgress).await.expect("Change status thất bại");
    assert_eq!(changed.status, TaskStatus::InProgress);
    assert_eq!(changed.sync_status, "pending");

    // 4. Cập nhật task (update_task)
    let update_input = UpdateTaskInput {
        id: created.id.clone(),
        title: "Học Rust, Egui và Supabase Sync".to_string(),
        description: Some("Cập nhật mô tả chi tiết hơn".to_string()),
        status: TaskStatus::InProgress,
        priority: Priority::High,
        due_date: Some(1735689600000),
        start_time: Some(1735690000000),
        end_time: Some(1735695000000),
        is_all_day: false,
        recurrence_rule: None,
    };

    let updated = repo.update_task(update_input).await.expect("Cập nhật task thất bại");
    assert_eq!(updated.title, "Học Rust, Egui và Supabase Sync");
    assert_eq!(updated.status, TaskStatus::InProgress);
    assert_eq!(updated.sync_status, "pending");
    assert!(updated.updated_at >= toggled.updated_at);

    // Test search_tasks với tiếng Việt không dấu
    let search_results = repo.search_tasks("hoc rust").await.expect("Tìm kiếm task thất bại");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, created.id);

    let search_desc = repo.search_tasks("chi tiet hon").await.expect("Tìm kiếm theo mô tả thất bại");
    assert_eq!(search_desc.len(), 1);
    assert_eq!(search_desc[0].id, created.id);

    // 5. Xóa mềm (soft_delete_task)
    repo.soft_delete_task(&created.id).await.expect("Xóa mềm task thất bại");

    // Task không còn xuất hiện trong list_active_tasks
    let active_after_delete = repo.list_active_tasks().await.expect("Lấy danh sách sau xóa");
    assert_eq!(active_after_delete.len(), 0);

    // Kiểm tra trực tiếp task trong DB: deleted_at và sync_status
    let row = sqlx::query("SELECT deleted_at, sync_status FROM tasks WHERE id = $1")
        .bind(&created.id)
        .fetch_one(repo.pool())
        .await
        .expect("Truy vấn trực tiếp task bị xóa mềm");

    use sqlx::Row;
    let deleted_at: Option<i64> = row.get("deleted_at");
    let sync_status: String = row.get("sync_status");

    assert!(deleted_at.is_some());
    assert_eq!(sync_status, "pending");
}

#[tokio::test]
async fn test_deserialize_and_upsert_supabase_task() {
    let json_str = r#"{
        "id": "01a20817-3654-7698-937b-5637aa9652b6",
        "user_id": "fdeaf58c-4352-4949-9507-76c9809f16aa",
        "title": "Tạo US và các AC",
        "description": "-Tạo tất cả các US và AC cho siêu trí nhớ",
        "status": "todo",
        "priority": 3,
        "due_date": 1795431600000,
        "start_time": 1795397400000,
        "end_time": 1795431600000,
        "created_at": 1795432068692,
        "updated_at": 1795432507157,
        "deleted_at": null
    }"#;

    let task: rust_task_mgr_lib::domain::task::Task =
        serde_json::from_str(json_str).expect("Deserialize Supabase task without sync_status must succeed");
    assert_eq!(task.sync_status, "synced");
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.status, TaskStatus::Todo);

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("test_sync.db");
    let repo = SqliteTaskRepo::new_with_path(&db_path)
        .await
        .expect("init repo");

    // Giả lập user đã đăng nhập
    let session = rust_task_mgr_lib::domain::auth::AuthSession {
        user_id: "fdeaf58c-4352-4949-9507-76c9809f16aa".to_string(),
        email: "ngocthientcvan.b10@gmail.com".to_string(),
        access_token: "test_token".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_at: 2000000000,
    };
    repo.save_session(&session).await.expect("save session");

    // Upsert task được pull về từ cloud
    let inserted = repo.upsert_pulled_task(&task).await.expect("upsert task");
    assert!(inserted);

    // Kiểm tra task xuất hiện trong list_active_tasks
    let active_tasks = repo.list_active_tasks().await.expect("list_active_tasks");
    assert_eq!(active_tasks.len(), 1);
    assert_eq!(active_tasks[0].id, "01a20817-3654-7698-937b-5637aa9652b6");
    assert_eq!(active_tasks[0].title, "Tạo US và các AC");
    assert_eq!(active_tasks[0].sync_status, "synced");
}

#[tokio::test]
async fn test_sync_engine_fallback_when_offline() {
    use std::sync::Arc;
    use rust_task_mgr_lib::sync::{SupabaseAuthClient, SyncEngine};

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("test_fallback.db");
    let repo = Arc::new(
        SqliteTaskRepo::new_with_path(&db_path)
            .await
            .expect("init repo"),
    );

    let auth_client = SupabaseAuthClient::new();
    let (engine, _rx) = SyncEngine::new(repo.clone(), auth_client);

    // Khi chưa cấu hình / chưa đăng nhập, get_tasks_online_or_offline fallback về SQLite
    let tasks = engine.get_tasks_online_or_offline().await.expect("fallback success");
    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn test_all_day_and_recurring_task_spawning() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = temp_dir.path().join("test_recurring.db");
    let repo = SqliteTaskRepo::new_with_path(&db_path)
        .await
        .expect("init repo");

    // Tạo task cả ngày lặp lại hàng ngày
    let base_start = 1795400000000_i64;
    let base_end = base_start + 86_399_999;
    let created = repo
        .create_task(CreateTaskInput {
            title: "Tập thể dục buổi sáng".to_string(),
            description: None,
            priority: Priority::Medium,
            due_date: Some(base_end),
            start_time: Some(base_start),
            end_time: Some(base_end),
            is_all_day: true,
            recurrence_rule: Some("daily".to_string()),
        })
        .await
        .expect("create recurring task");

    assert!(created.is_all_day);
    assert_eq!(created.recurrence_rule.as_deref(), Some("daily"));

    // Hoàn thành kỳ hiện tại -> phải tự động sinh kỳ tiếp theo (+1 ngày)
    let done_task = repo.toggle_status(&created.id).await.expect("toggle done");
    assert_eq!(done_task.status, TaskStatus::Done);

    let all_tasks = repo.list_active_tasks().await.expect("list tasks");
    assert_eq!(all_tasks.len(), 2);

    let next_occ = all_tasks
        .iter()
        .find(|t| t.id != created.id)
        .expect("next occurrence must exist");
    assert_eq!(next_occ.status, TaskStatus::Todo);
    assert!(next_occ.is_all_day);
    assert_eq!(next_occ.recurrence_rule.as_deref(), Some("daily"));
    assert_eq!(next_occ.start_time, Some(base_start + 86_400_000));
    assert_eq!(next_occ.end_time, Some(base_end + 86_400_000));
}

#[test]
fn test_local_reminder_trigger_logic() {
    use std::collections::HashMap;
    use rust_task_mgr_lib::domain::task::Task;
    use rust_task_mgr_lib::reminder::should_notify_task;

    let mut task = Task::new_v7("Đặt bánh kem".to_string(), None, Priority::Medium, None);
    let scheduled_at = 1795420800000_i64; // e.g. 15:00:00
    task.start_time = Some(scheduled_at);
    task.end_time = Some(scheduled_at + 3600_000);
    task.due_date = Some(scheduled_at);

    let mut notified: HashMap<String, i64> = HashMap::new();

    // Trước giờ hẹn 1 phút -> chưa thông báo
    assert_eq!(should_notify_task(&task, scheduled_at - 60_000, &notified), None);

    // Đúng giờ hẹn (hoặc trễ 5 giây) -> gửi thông báo
    assert_eq!(should_notify_task(&task, scheduled_at + 5_000, &notified), Some(scheduled_at));

    // Đã đánh dấu đã thông báo mốc scheduled_at -> không gửi lặp lại
    notified.insert(task.id.clone(), scheduled_at);
    assert_eq!(should_notify_task(&task, scheduled_at + 15_000, &notified), None);

    // Nếu dời lịch sang mốc mới -> thông báo lại ở mốc mới
    let new_scheduled = scheduled_at + 3600_000;
    task.start_time = Some(new_scheduled);
    task.end_time = Some(new_scheduled + 3600_000);
    task.due_date = Some(new_scheduled);
    assert_eq!(should_notify_task(&task, new_scheduled + 2_000, &notified), Some(new_scheduled));
}
