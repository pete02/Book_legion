use std::env;

use reqwest::{Client, Url};
use serde_json::Value;

pub async fn refresh_auth_token() -> Result<String, String> {
    let bearer_token = env::var("TRIBUNE_LOGISTICA_API_TOKEN")
        .map_err(|_| "missing env: TRIBUNE_LOGISTICA_API_TOKEN")?;

    let body = serde_json::json!({ "refresh_token": bearer_token });
    let url = Url::parse("https://staging.lumilukko.com/api/v1/refreshtoken")
        .map_err(|_| "Url const is wrong".to_string())?;

    let client = Client::new();

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body) // handles serde_json serialization
        .send()
        .await
        .map_err(|e| e.to_string())?;


    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let new_token = json["auth_token"]
        .as_str()
        .ok_or("Invalid response")?;
    Ok(new_token.to_string())
}


pub async fn post_new_book(auth_token: &str, data: &Value)->Result<(),String>{
    let url = Url::parse("https://staging.lumilukko.com/api/v1/savebook")
        .map_err(|_| "Url const is wrong".to_string())?;

    let client = Client::new();

    let _resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .bearer_auth(auth_token)
        .json(data) // handles serde_json serialization
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}