use crate::{error::AppError, state::AppState};
use argon2::password_hash::{Error as PasswordHashError, PasswordHash, PasswordVerifier};
use argon2::Argon2;
use axum::{
    extract::State,
    http::{header, HeaderMap},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/login", post(login_handler))
        .route("/api/auth/status", get(auth_status_handler))
        .route("/api/logout", post(logout_handler))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub ok: bool,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), AppError> {
    tracing::info!(
        "login attempt with password len = {}",
        payload.password.len()
    );

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&state.config.auth.password_hash)
        .map_err(|_| AppError::Internal("invalid password hash".into()))?;

    if let Err(err) = argon2.verify_password(payload.password.as_bytes(), &parsed_hash) {
        return match err {
            PasswordHashError::Password => Err(AppError::Unauthorized),
            _ => Err(AppError::Internal("password verification failed".into())),
        };
    }

    let session_id = state.sessions.create_session().await;
    let ttl_seconds = state
        .config
        .server
        .session_timeout_minutes
        .saturating_mul(60);
    let cookie_value = format!(
        "session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}",
        session_id, ttl_seconds
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&cookie_value)
            .map_err(|_| AppError::Internal("failed to set session cookie".into()))?,
    );

    Ok((headers, Json(LoginResponse { ok: true })))
}

pub async fn auth_status_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthStatusResponse>, AppError> {
    // Extract session ID from cookies
    let session_id = extract_session_id_from_headers(&headers);

    let authenticated = if let Some(session_id) = session_id {
        state.sessions.validate(&session_id).await
    } else {
        false
    };

    Ok(Json(AuthStatusResponse { authenticated }))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<LogoutResponse>), AppError> {
    // Extract session ID from cookies
    let session_id = extract_session_id_from_headers(&headers);

    if let Some(session_id) = session_id {
        state.sessions.remove(&session_id).await;
    }

    // Create cookie header to clear the session cookie
    let cookie_value = "session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0";
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&cookie_value)
            .map_err(|_| AppError::Internal("failed to clear session cookie".into()))?,
    );

    Ok((headers, Json(LogoutResponse { ok: true })))
}

// Helper function to extract session ID from cookies
fn extract_session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        let cookies = cookie_header.to_str().ok().unwrap_or("").split(';');
        for cookie in cookies {
            let parts: Vec<&str> = cookie.trim().split('=').collect();
            if parts.len() == 2 && parts[0].trim() == "session" {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}
