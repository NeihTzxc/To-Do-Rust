use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl<'de> Deserialize<'de> for TaskStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(TaskStatus::from_str_loose(&s))
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr)]
#[repr(i64)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PriorityVisitor;
        impl<'de> serde::de::Visitor<'de> for PriorityVisitor {
            type Value = Priority;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer or string representing priority")
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Priority::from_i64(v))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Priority::from_i64(v as i64))
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.to_lowercase().as_str() {
                    "high" | "3" => Ok(Priority::High),
                    "low" | "1" => Ok(Priority::Low),
                    _ => Ok(Priority::Medium),
                }
            }
        }
        deserializer.deserialize_any(PriorityVisitor)
    }
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

fn default_sync_status() -> String {
    "synced".to_string()
}

fn default_now() -> i64 {
    Utc::now().timestamp_millis()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub user_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    #[serde(default)]
    pub is_all_day: bool,
    pub recurrence_rule: Option<String>,
    #[serde(default = "default_now")]
    pub created_at: i64,
    #[serde(default = "default_now")]
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    #[serde(default = "default_sync_status")]
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
            user_id: None,
            title,
            description,
            status: TaskStatus::Todo,
            priority,
            due_date,
            start_time: None,
            end_time: None,
            is_all_day: false,
            recurrence_rule: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            sync_status: "pending".to_string(),
        }
    }
}

/// Tính thời điểm (epoch ms) cho kỳ tiếp theo dựa trên `base_ms` và `recurrence_rule`.
/// Hỗ trợ: "daily", "weekly", "monthly", hoặc "weekly:1,3,5" (1=T2 .. 7=CN).
pub fn compute_next_occurrence_ms(base_ms: i64, rule: &str) -> Option<i64> {
    use chrono::{Datelike, Duration, Local, Months, TimeZone};

    let trimmed = rule.trim().to_lowercase();
    if trimmed.is_empty() {
        return None;
    }

    let dt = Local.timestamp_millis_opt(base_ms).single()?;

    if trimmed == "daily" {
        return Some((dt + Duration::days(1)).timestamp_millis());
    }
    if trimmed == "weekly" {
        return Some((dt + Duration::days(7)).timestamp_millis());
    }
    if trimmed == "monthly" {
        let next = dt
            .checked_add_months(Months::new(1))
            .unwrap_or_else(|| dt + Duration::days(30));
        return Some(next.timestamp_millis());
    }
    if let Some(days_part) = trimmed.strip_prefix("weekly:") {
        let target_days: Vec<u32> = days_part
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .filter(|&d| (1..=7).contains(&d))
            .collect();
        if target_days.is_empty() {
            return Some((dt + Duration::days(7)).timestamp_millis());
        }
        for offset in 1..=7 {
            let candidate = dt + Duration::days(offset);
            let wd = candidate.weekday().number_from_monday();
            if target_days.contains(&wd) {
                return Some(candidate.timestamp_millis());
            }
        }
    }

    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub due_date: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    #[serde(default)]
    pub is_all_day: bool,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    #[serde(default)]
    pub is_all_day: bool,
    pub recurrence_rule: Option<String>,
}
