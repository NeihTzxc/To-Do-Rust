use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusState {
    pub is_logged_in: bool,
    pub user_email: Option<String>,
    pub state: String,
    pub pending_count: i64,
    pub last_synced_at: Option<i64>,
    pub error_message: Option<String>,
}

