use std::sync::Arc;
use tauri::State;

use crate::domain::auth::{SyncStatusState, UserInfo};
use crate::domain::task::{CreateTaskInput, Priority, Task, TaskStatus, UpdateTaskInput};
use crate::storage::TaskRepository;
use crate::sync::{SupabaseAuthClient, SyncEngine};

#[tauri::command]
pub async fn get_tasks(
    state: State<'_, Arc<dyn TaskRepository>>,
) -> Result<Vec<Task>, String> {
    state.list_active_tasks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_task(
    state: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    title: String,
    description: Option<String>,
    priority: Option<i64>,
    due_date: Option<i64>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<Task, String> {
    let priority_val = priority.map(Priority::from_i64).unwrap_or(Priority::Medium);
    let input = CreateTaskInput {
        title,
        description,
        priority: priority_val,
        due_date,
        start_time,
        end_time,
    };
    let task = state.create_task(input).await.map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(task)
}

#[tauri::command]
pub async fn update_task(
    state: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    id: String,
    title: String,
    description: Option<String>,
    status: String,
    priority: i64,
    due_date: Option<i64>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<Task, String> {
    let input = UpdateTaskInput {
        id,
        title,
        description,
        status: TaskStatus::from_str_loose(&status),
        priority: Priority::from_i64(priority),
        due_date,
        start_time,
        end_time,
    };
    let task = state.update_task(input).await.map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(task)
}

#[tauri::command]
pub async fn update_task_schedule(
    state: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    id: String,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<Task, String> {
    let task = state
        .update_schedule(&id, start_time, end_time)
        .await
        .map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(task)
}

#[tauri::command]
pub async fn toggle_task_status(
    state: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    id: String,
) -> Result<Task, String> {
    let task = state.toggle_status(&id).await.map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(task)
}

#[tauri::command]
pub async fn move_task_status(
    state: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    id: String,
    status: String,
) -> Result<Task, String> {
    let new_status = TaskStatus::from_str_loose(&status);
    let task = state.change_status(&id, new_status).await.map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(task)
}

#[tauri::command]
pub async fn delete_task(
    state: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    id: String,
) -> Result<(), String> {
    state.soft_delete_task(&id).await.map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(())
}

#[tauri::command]
pub async fn search_tasks(
    state: State<'_, Arc<dyn TaskRepository>>,
    query: String,
) -> Result<Vec<Task>, String> {
    state.search_tasks(&query).await.map_err(|e| e.to_string())
}

// --- Auth & User Commands ---

#[tauri::command]
pub async fn login(
    repo: State<'_, Arc<dyn TaskRepository>>,
    auth_client: State<'_, Arc<SupabaseAuthClient>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    email: String,
    password: String,
) -> Result<UserInfo, String> {
    let session = auth_client
        .login(&email, &password, &repo)
        .await
        .map_err(|e| e.to_string())?;

    sync_engine.trigger_sync();

    Ok(UserInfo {
        user_id: session.user_id,
        email: session.email,
    })
}

#[tauri::command]
pub async fn register(
    repo: State<'_, Arc<dyn TaskRepository>>,
    auth_client: State<'_, Arc<SupabaseAuthClient>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
    email: String,
    password: String,
) -> Result<UserInfo, String> {
    let session = auth_client
        .signup(&email, &password, &repo)
        .await
        .map_err(|e| e.to_string())?;

    sync_engine.trigger_sync();

    Ok(UserInfo {
        user_id: session.user_id,
        email: session.email,
    })
}

#[tauri::command]
pub async fn logout(
    repo: State<'_, Arc<dyn TaskRepository>>,
    sync_engine: State<'_, Arc<SyncEngine>>,
) -> Result<(), String> {
    repo.clear_session().await.map_err(|e| e.to_string())?;
    sync_engine.trigger_sync();
    Ok(())
}

#[tauri::command]
pub async fn get_current_user(
    repo: State<'_, Arc<dyn TaskRepository>>,
) -> Result<Option<UserInfo>, String> {
    repo.get_current_user().await.map_err(|e| e.to_string())
}

// --- Sync Commands ---

#[tauri::command]
pub async fn trigger_sync_now(
    sync_engine: State<'_, Arc<SyncEngine>>,
) -> Result<(), String> {
    sync_engine.trigger_sync();
    Ok(())
}

#[tauri::command]
pub async fn get_sync_status(
    sync_engine: State<'_, Arc<SyncEngine>>,
) -> Result<SyncStatusState, String> {
    Ok(sync_engine.get_status().await)
}
