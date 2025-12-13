use std::collections::HashMap;

use dioxus::logger::tracing;


use crate::models::{BookStatus, ChunkData, ChunkProgress, RefreshRecord, Tokens};

use crate::components::audio::ADVANCE_AMOUNT;

pub async fn fetch_audiomap(book: &BookStatus, access_token:String) -> Result<HashMap<String,ChunkProgress>, Box<dyn std::error::Error>> {
    let data=reqwasm::http::Request::post("http://127.0.0.1:8000/audiomap")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", access_token))
        .body(serde_json::to_string(&book)?)
        .send()
        .await?.text().await?;


    let chunkdata:ChunkData=serde_json::from_str(&data)?;
    let map: HashMap<String, ChunkProgress>=chunkdata.data.map;
    return Ok(map)
}

pub async fn fetch_audio(book: &BookStatus, access_token:String) -> Result<(bool,Vec<u8>), Box<dyn std::error::Error>> {
    let mut book=book.clone();
    book.chapter=book.chapter.clamp(book.initial_chapter, book.max_chapter);
    book.chunk=book.chunk.clamp(1, book.chapter_to_chunk[&book.chapter]);

    let url = format!("http://127.0.0.1:8000/audio?chunk={}", ADVANCE_AMOUNT);
    let bytes = reqwasm::http::Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", access_token))
        .body(serde_json::to_string(&book)?)
        .send()
        .await?;
    match bytes.headers().get("reached-end"){
        Some(value)=> Ok((value == "true", bytes.binary().await?)),
        None=>return Err("incorrect headers".into())
    }
}

pub async fn update_progress(book:BookStatus, access_token:String)->Result<(),Box<dyn std::error::Error>>{
    let _=reqwasm::http::Request::post("http://127.0.0.1:8000/update")
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", access_token))
        .body(serde_json::to_string(&book)?)
        .send()
        .await?.text().await?;

    Ok(())
}


pub async fn fetch_chapter(book: BookStatus, access_token:String) -> Result<String, Box<dyn std::error::Error>> {
    let url = "http://127.0.0.1:8000/book";
    let bytes = reqwasm::http::Request::post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", access_token))
        .body(serde_json::to_string(&book)?)
        .send()
        .await?;
    let text = bytes.text().await?;
    Ok(text)
}

pub async fn get_book(book_name:String, access_token:String) -> Result<BookStatus, Box<dyn std::error::Error>> {
    let json: BookStatus = reqwasm::http::Request::get(&format!("http://127.0.0.1:8000/init?name={}&type=text",book_name))
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await?
        .json()
        .await?;
    Ok(json)
}

pub async fn fetch_css(book: &str, access_token:String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:8000/css/{}", book);
    let resp = reqwasm::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}",access_token))
        .send().await?;
    
    if resp.status() >= 500 {
        tracing::error!("Backend error: {:?}", resp.text().await);
    }
    
    let text = resp.text().await?;
    Ok(text)
}



pub async fn fetch_login(
    user: &str,
    pass: &str,
) -> Result<Tokens, Box<dyn std::error::Error>> {
    let url = "http://127.0.0.1:8000/login";

    let body = serde_json::json!({
        "username": user,
        "password": pass
    });

    let resp = reqwasm::http::Request::post(url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?;

    if !resp.ok() {
        return Err(format!("Login failed: {}", resp.status()).into());
    }

    let text = resp.text().await?;
    let tokens:Tokens=serde_json::from_str(&text)?;
    Ok(tokens)
}

pub async fn refresh_access_token(user:String, refresh_token:String)->Result<Tokens, Box<dyn std::error::Error>>{
    let record=RefreshRecord::new(user, refresh_token);
    let resp=reqwasm::http::Request::post("http://127.0.0.1:8000/refresh")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&record)?)
        .send().await?;
    let txt=resp.text().await?;
    let tokens:Tokens=serde_json::from_str(&txt)?;
    Ok(tokens)
}

pub async fn fetch_manifest(access_token:String)-> Result<String, Box<dyn std::error::Error>>{
    tracing::debug!("start manifest");
    tracing::debug!("access token got");
    let url = format!("http://127.0.0.1:8000/manifest");
    let resp = reqwasm::http::Request::get(&url)
        .header("Authorization", &format!("Bearer {}",access_token))
        .send().await?;
    let text = resp.text().await?;
    Ok(text)
}