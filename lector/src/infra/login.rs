use serde::{Deserialize, Serialize};
use dioxus::prelude::*;
#[cfg(feature = "mock")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "mock")]
use std::sync::Arc;

#[derive(Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub auth_token: String,
    pub refresh_token: String,
}

#[cfg(feature = "mock")]
static LOGIN_COUNTER: once_cell::sync::Lazy<Arc<AtomicUsize>> =
    once_cell::sync::Lazy::new(|| Arc::new(AtomicUsize::new(1)));

#[cfg(not(feature = "mock"))]
pub async fn login(username: &str, password: &str) -> Result<LoginResponse, Box<dyn std::error::Error>> {
    use crate::infra::auth::post_with_auth;

    let body = LoginRequest { username, password };
    let resp = post_with_auth("/api/v1/login",serde_json::to_string(&body)?)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("{}: {}", status, text).into());
    }

    let login_resp = resp
        .json::<LoginResponse>()
        .await
        .map_err(|e| e.to_string())?;
    Ok(login_resp)
}

/// Mock login simulates odd/even login attempts
#[cfg(feature = "mock")]
pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    use dioxus::logger::tracing;
    

    if username.is_empty() || password.is_empty() {
        return Err("username and password must be provided".into());
    }
    let counter = LOGIN_COUNTER.fetch_add(1, Ordering::SeqCst);

    /* 
    if counter % 2 == 1 {
        tracing::debug!("deny login");
        return Err("mock login failed on odd attempt".into());
    }
    */

    tracing::debug!("accept login");
    Ok(LoginResponse {
        auth_token: "mock_auth_token".into(),
        refresh_token: "mock_refresh_token".into(),
    })
}
