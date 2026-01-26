use serde::{Deserialize, Serialize};

use crate::domain;
use crate::domain::cursor::BookCursor;


#[derive(Debug, Clone, Serialize,Deserialize)]
pub struct GetChunksRequest {
    #[serde(rename = "UserCursor")]
    pub user_cursor: BookCursor,
    #[serde(rename = "requestSize")]
    pub request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResponse {
    pub data:  Vec<u8>,
    #[serde(rename = "Cursor")]
    pub cursor: domain::cursor::Cursor,
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

    let start_chapter = cursor.cursor.chapter;
    let start_chunk = cursor.cursor.chunk;

    let mut results = Vec::with_capacity(request_size);

    sleep(Duration::from_millis(1000)).await;

    for i in 0..request_size {
        results.push(ChunkResponse {
            data: bytes.to_vec(),
            cursor: domain::cursor::Cursor {
                chapter: start_chapter,
                chunk: start_chunk + i as usize,
            },
        });
    }

    Ok(results)
}