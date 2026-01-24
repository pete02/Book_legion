use reqwasm::http::Request;

use crate::domain;

pub async fn get_with_auth(
    url: &str
) -> Result<reqwasm::http::Response, String> {
    let auth_token = domain::login::current_auth();
    let refresh_token = domain::login::current_refresh();

    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", auth_token.unwrap_or_default()))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() != 401 {
        return Ok(resp);
    }

    let refresh_token = refresh_token.ok_or("No refresh token available")?;
    let new_auth = refresh_auth_token(&refresh_token).await?;
    domain::login::set_auth(Some(new_auth.clone()));

    let retry_resp = Request::get(url)
        .header("Authorization", &format!("Bearer {}", new_auth))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if retry_resp.status() == 401 {
        domain::login::set_refresh(None);
        return Err("Unauthorized after refresh".into());
    }

    Ok(retry_resp)
}

pub async fn post_with_auth(
    url: &str,
    body: String,
) -> Result<reqwasm::http::Response, String> {
    let auth_token = domain::login::current_auth();
    let refresh_token = domain::login::current_refresh();

    let resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", auth_token.unwrap_or_default()))
        .header("Content-Type", "application/json")
        .body(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() != 401 {
        return Ok(resp);
    }

    let refresh_token = refresh_token.ok_or("No refresh token available")?;
    let new_auth = refresh_auth_token(&refresh_token).await?;
    domain::login::set_auth(Some(new_auth.clone()));

    let retry_resp = Request::post(url)
        .header("Authorization", &format!("Bearer {}", new_auth))
        .header("Content-Type", "application/json")
        .body(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if retry_resp.status() == 401 {
        domain::login::set_refresh(None);
        return Err("Unauthorized after refresh".into());
    }

    Ok(retry_resp)
}
async fn refresh_auth_token(refresh_token: &str) -> Result<String, String> {
    let body = serde_json::json!({ "refresh_token": refresh_token });
    let resp = Request::post("/api/v1/refreshtoken")
        .body(serde_json::to_string(&body).unwrap())
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        domain::login::set_refresh(None);
        return Err("Failed to refresh token".into());
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let new_token = json["auth_token"]
        .as_str()
        .ok_or("Invalid response")?;
    Ok(new_token.to_string())
}