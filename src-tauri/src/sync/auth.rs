use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::auth::AuthSession;
use crate::storage::TaskRepository;

#[derive(Clone)]
pub struct SupabaseAuthClient {
    client: reqwest::Client,
    supabase_url: String,
    anon_key: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SupabaseAuthResponse {
    access_token: String,
    token_type: Option<String>,
    expires_in: Option<i64>,
    refresh_token: String,
    user: SupabaseUser,
}

#[derive(Deserialize)]
struct SupabaseUser {
    id: String,
    email: Option<String>,
}

#[derive(Serialize)]
struct AuthPayload<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct RefreshPayload<'a> {
    refresh_token: &'a str,
}

impl SupabaseAuthClient {
    pub fn new() -> Self {
        // Ưu tiên đọc từ biến môi trường runtime (sau khi load dotenvy), fallback sang biến nhúng lúc compile
        let supabase_url = std::env::var("SUPABASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| option_env!("SUPABASE_URL").map(|s| s.to_string()))
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_string();

        let anon_key = std::env::var("SUPABASE_ANON_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| option_env!("SUPABASE_ANON_KEY").map(|s| s.to_string()))
            .unwrap_or_default()
            .trim()
            .to_string();

        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            supabase_url,
            anon_key,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.supabase_url.is_empty()
            && !self.anon_key.is_empty()
            && !self.supabase_url.contains("your-project-ref")
    }

    pub fn url(&self) -> &str {
        &self.supabase_url
    }

    pub fn anon_key(&self) -> &str {
        &self.anon_key
    }

    /// Đăng ký tài khoản mới bằng Email & Mật khẩu thông qua Supabase GoTrue API
    pub async fn signup(
        &self,
        email: &str,
        password: &str,
        repo: &Arc<dyn TaskRepository>,
    ) -> Result<AuthSession> {
        if !self.is_configured() {
            bail!("Supabase chưa được cấu hình! Vui lòng thiết lập SUPABASE_URL và SUPABASE_ANON_KEY trong file .env.");
        }

        let endpoint = format!("{}/auth/v1/signup", self.supabase_url);
        let resp = self
            .client
            .post(&endpoint)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&AuthPayload { email, password })
            .send()
            .await
            .context("Không thể kết nối tới máy chủ Supabase Auth")?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            bail!("Đăng ký thất bại: {}", err_body);
        }

        let data: SupabaseAuthResponse = resp
            .json()
            .await
            .context("Lỗi phân tích cú pháp phản hồi từ Supabase Auth")?;

        let now_sec = Utc::now().timestamp();
        let expires_at = now_sec + data.expires_in.unwrap_or(3600);

        let session = AuthSession {
            user_id: data.user.id,
            email: data.user.email.unwrap_or_else(|| email.to_string()),
            access_token: data.access_token,
            refresh_token: data.refresh_token,
            expires_at,
        };

        repo.save_session(&session).await?;
        Ok(session)
    }

    /// Đăng nhập bằng Email & Mật khẩu
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        repo: &Arc<dyn TaskRepository>,
    ) -> Result<AuthSession> {
        if !self.is_configured() {
            bail!("Supabase chưa được cấu hình! Vui lòng thiết lập SUPABASE_URL và SUPABASE_ANON_KEY trong file .env.");
        }

        let endpoint = format!("{}/auth/v1/token?grant_type=password", self.supabase_url);
        let resp = self
            .client
            .post(&endpoint)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&AuthPayload { email, password })
            .send()
            .await
            .context("Không thể kết nối tới máy chủ Supabase Auth")?;

        if !resp.status().is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            bail!("Đăng nhập thất bại: {}", err_body);
        }

        let data: SupabaseAuthResponse = resp
            .json()
            .await
            .context("Lỗi đọc dữ liệu phiên đăng nhập từ Supabase")?;

        let now_sec = Utc::now().timestamp();
        let expires_at = now_sec + data.expires_in.unwrap_or(3600);

        let session = AuthSession {
            user_id: data.user.id,
            email: data.user.email.unwrap_or_else(|| email.to_string()),
            access_token: data.access_token,
            refresh_token: data.refresh_token,
            expires_at,
        };

        repo.save_session(&session).await?;
        Ok(session)
    }

    /// Lấy access_token hợp lệ. Nếu sắp hết hạn (còn dưới 60 giây), tự động làm mới bằng refresh_token
    pub async fn get_valid_access_token(
        &self,
        repo: &Arc<dyn TaskRepository>,
    ) -> Result<Option<String>> {
        let session = match repo.get_current_session().await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let now_sec = Utc::now().timestamp();
        // Nếu token còn hạn trên 60s -> trả về luôn
        if now_sec < session.expires_at - 60 {
            return Ok(Some(session.access_token));
        }

        // Token sắp hết hạn hoặc đã hết hạn -> gọi refresh_token
        if !self.is_configured() {
            return Ok(Some(session.access_token));
        }

        let endpoint = format!("{}/auth/v1/token?grant_type=refresh_token", self.supabase_url);
        let resp = self
            .client
            .post(&endpoint)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&RefreshPayload {
                refresh_token: &session.refresh_token,
            })
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(data) = r.json::<SupabaseAuthResponse>().await {
                    let expires_at = now_sec + data.expires_in.unwrap_or(3600);
                    let new_session = AuthSession {
                        user_id: data.user.id,
                        email: data.user.email.unwrap_or(session.email),
                        access_token: data.access_token.clone(),
                        refresh_token: data.refresh_token,
                        expires_at,
                    };
                    repo.save_session(&new_session).await?;
                    return Ok(Some(new_session.access_token));
                }
            }
            _ => {
                // Nếu refresh thất bại do mạng, trả về access_token cũ để thử tiếp
            }
        }

        Ok(Some(session.access_token))
    }
}
