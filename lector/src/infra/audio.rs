use dioxus::logger::tracing;
use serde::{Deserialize, Serialize,Deserializer};
use base64::{engine::general_purpose, Engine as _};


use crate::domain;
use crate::domain::cursor::BookCursor;

fn base64_to_vec<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    general_purpose::STANDARD
        .decode(s)
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResponse {
    #[serde(deserialize_with = "base64_to_vec")]
    pub data:  Vec<u8>,
    pub cursor: domain::cursor::BookCursor,
}

#[derive(Debug, Clone, Serialize,Deserialize)]
pub struct GetChunksRequest {
    pub user_cursor: BookCursor,
    #[serde(rename = "requestSize")]
    pub request_size: usize,
}



#[cfg(not(feature = "mock"))]
pub async fn get_chunks(
    cursor:BookCursor,
    request_size:usize
) -> Result<Vec<ChunkResponse>, Box<dyn std::error::Error>> {
    use crate::infra::auth::post_with_auth;
    let book_id=cursor.book_id.clone();
    let url = format!("/api/v1/books/{}/chunks", &book_id);
    let req=GetChunksRequest{
        user_cursor: cursor,
        request_size: request_size
    };

    let resp = post_with_auth(&url,serde_json::to_string(&req)?).await?;
    if !resp.ok() {
        return Err(format!(
            "Failed to get chunks for book {}: {}",
            &book_id,
            resp.status()
        )
        .into());
    }

    let chunks: Vec<ChunkResponse> =
        resp.json().await.map_err(|e| e.to_string())?;

    Ok(chunks)
}


#[cfg(feature = "mock")]
pub async fn get_chunks(
    cursor:BookCursor,
    request_size:usize
) -> Result<Vec<ChunkResponse>, Box<dyn std::error::Error>> {
    use std::time::Duration;

    use gloo_timers::future::sleep;

    use crate::assets;

    if cursor.book_id != "b1" {
        return Err("no chunks found".into());
    }

    let bytes = dioxus::asset_resolver::read_asset_bytes(&assets::MOCK_MP3).await.unwrap();
    let base64_string = general_purpose::STANDARD.encode(&bytes);
    let start_chapter = cursor.cursor.chapter;
    let start_chunk = cursor.cursor.chunk;

    let mut results = Vec::with_capacity(request_size);

    sleep(Duration::from_millis(1000)).await;
    
    for i in 0..request_size {
        let chunk_json = serde_json::json!({
            "cursor": {
                "user_id": cursor.user_id,
                "book_id": cursor.book_id,
                "cursor": {
                    "chapter": start_chapter,
                    "chunk": start_chunk + i
                }
            },
            "data": base64_string
        });
        results.push(chunk_json);
    }
    let json_value = serde_json::Value::Array(results);
    let json_string = serde_json::to_string(&json_value)?;

    let chunks: Vec<ChunkResponse> = serde_json::from_str(&json_string)
        .map_err(|e| format!("Failed to parse mocked chunks JSON: {}", e))?;

    Ok(chunks)
}