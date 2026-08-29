use std::env;

use log::{debug, error, info};
use reqwest::{Client, Url};
use serde_json::Value;

pub async fn refresh_auth_token() -> Result<String, String> {
    let base_url=env::var("TRIBUNE_LOGISTICA_URL")
        .map_err(|_| "missing env: TRIBUNE_LOGISTICA_URL")?;
    let user=env::var("TRIBUNE_LOGISTICA_USER")
        .map_err(|_| "missing env: TRIBUNE_LOGISTICA_USER")?;
    let password=env::var("TRIBUNE_LOGISTICA_PASSWORD")
        .map_err(|_| "missing env: TRIBUNE_LOGISTICA_PASSWORD")?;
    let body = serde_json::json!({ "username": user, "password": password });

    let url = Url::parse(&format!("{}/api/v1/login",base_url))
        .map_err(|_| "Url const is wrong".to_string())?;

    let client = Client::new();
    debug!("refreshing from {}", url);

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body) // handles serde_json serialization
        .send()
        .await
        .map_err(|e| {
            error!("Error in refreshing: {:?}", e);
            e.to_string()
        })?;
    info!("Response: {}", resp.status());
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        
    let new_token = json["auth_token"]
        .as_str()
        .ok_or("Invalid response")?;
    Ok(new_token.to_string())
}


pub async fn post_new_book(auth_token: &str, data: &Value)->Result<(),String>{

    let test = env::var("DEBUG")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if test {return Ok(());}

    let base_url=env::var("TRIBUNE_LOGISTICA_URL")
        .map_err(|_| "missing env: TRIBUNE_LOGISTICA_URL")?;

    let url = Url::parse(&format!("{}/api/v1/savebook",base_url))
        .map_err(|_| "Url const is wrong".to_string())?;
    debug!("posting to {}", url);
    debug!(" with: {}", data);

    let client = Client::new();

    let _resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .bearer_auth(auth_token)
        .json(data) // handles serde_json serialization
        .send()
        .await
        .map_err(|e| {
            error!("Error in saving book: {}",e);
            e.to_string()
        })?;

    Ok(())
}