use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "Cần làm",
            TaskStatus::InProgress => "Đang làm",
            TaskStatus::Done => "Đã xong",
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self, TaskStatus::Done)
    }

    pub fn toggle(&self) -> Self {
        match self {
            TaskStatus::Done => TaskStatus::Todo,
            TaskStatus::Todo | TaskStatus::InProgress => TaskStatus::Done,
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "in_progress" | "inprogress" | "doing" => TaskStatus::InProgress,
            "done" | "completed" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(i64)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl Priority {
    pub fn as_i64(&self) -> i64 {
        *self as i64
    }

    pub fn from_i64(val: i64) -> Self {
        match val {
            1 => Priority::Low,
            3 => Priority::High,
            _ => Priority::Medium,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Priority::Low => "Thấp (Low)",
            Priority::Medium => "Vừa (Medium)",
            Priority::High => "Cao (High)",
        }
    }

    pub fn short_badge(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub sync_status: String,
}

impl Task {
    #[allow(dead_code)]
    pub fn new_v7(
        title: String,
        description: Option<String>,
        priority: Priority,
        due_date: Option<i64>,
    ) -> Self {
        let now = Utc::now().timestamp_millis();
        let id = Uuid::now_v7().to_string();
        Self {
            id,
            title,
            description,
            status: TaskStatus::Todo,
            priority,
            due_date,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            sync_status: "pending".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub due_date: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<i64>,
}
