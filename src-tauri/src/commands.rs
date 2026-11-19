use std::sync::Arc;
use tauri::State;

use crate::domain::task::{CreateTaskInput, Priority, Task, TaskStatus, UpdateTaskInput};
use crate::storage::TaskRepository;

#[tauri::command]
pub async fn get_tasks(
    state: State<'_, Arc<dyn TaskRepository>>,
) -> Result<Vec<Task>, String> {
    state.list_active_tasks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_task(
    state: State<'_, Arc<dyn TaskRepository>>,
    title: String,
    description: Option<String>,
    priority: Option<i64>,
    due_date: Option<i64>,
) -> Result<Task, String> {
    let priority_val = priority.map(Priority::from_i64).unwrap_or(Priority::Medium);
    let input = CreateTaskInput {
        title,
        description,
        priority: priority_val,
        due_date,
    };
    state.create_task(input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    state: State<'_, Arc<dyn TaskRepository>>,
    id: String,
    title: String,
    description: Option<String>,
    status: String,
    priority: i64,
    due_date: Option<i64>,
) -> Result<Task, String> {
    let input = UpdateTaskInput {
        id,
        title,
        description,
        status: TaskStatus::from_str_loose(&status),
        priority: Priority::from_i64(priority),
        due_date,
    };
    state.update_task(input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_task_status(
    state: State<'_, Arc<dyn TaskRepository>>,
    id: String,
) -> Result<Task, String> {
    state.toggle_status(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_task_status(
    state: State<'_, Arc<dyn TaskRepository>>,
    id: String,
    status: String,
) -> Result<Task, String> {
    let new_status = TaskStatus::from_str_loose(&status);
    state.change_status(&id, new_status).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(
    state: State<'_, Arc<dyn TaskRepository>>,
    id: String,
) -> Result<(), String> {
    state.soft_delete_task(&id).await.map_err(|e| e.to_string())
}
