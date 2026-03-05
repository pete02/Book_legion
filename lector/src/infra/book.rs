#[cfg(feature = "mock")]
use std::sync::Mutex;

use crate::infra::auth::{delete_with_auth, get_with_auth};
#[cfg(feature = "mock")]
use dioxus::prelude::info;
#[cfg(feature = "mock")]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookInfo {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub series_id: String,
    pub series_order: usize,
    pub file_path: String,
}
impl BookInfo {
    pub fn new()->BookInfo{
        BookInfo{
            id: "".to_string(),
            title: "".to_string(),
            author_id: "".to_string(),
            series_id: "".to_string(),
            series_order: 0,
            file_path: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProgressResponse {
    pub progress: f64,
}
#[cfg(not(feature = "mock"))]
pub async fn delete_book(book_id: &str)-> Result<(),String>{
    let url = format!("/api/v1/deletebook/{}", book_id);
    let resp=delete_with_auth(&url).await;
    if resp.is_ok(){
        return Ok(());
    }else{
        return Err(resp.unwrap_err());
    }

}

#[cfg(feature = "mock")]
pub async fn delete_book(book_id: &str)-> Result<(),String>{
    info!("delete a booK: {}", book_id);
    Ok(())
}


#[cfg(not(feature = "mock"))]
pub async fn save_book(book: &BookInfo) -> Result<(), String> {
    use crate::infra::auth::post_with_auth;

    let url = "/api/v1/savebook";
    let body = serde_json::json!({
        "id": book.id,
        "title": book.title,
        "author_id": book.author_id,
        "series_id": book.series_id,
        "series_order": book.series_order,
        "series_name": "",
        "file_path": book.file_path,
    }).to_string();

    let resp = post_with_auth(url, body).await;
    if resp.is_ok() {
        Ok(())
    } else {
        Err(resp.unwrap_err())
    }
}
#[cfg(feature = "mock")]
pub async fn save_book(book: &BookInfo) -> Result<(), String> {
    info!("save book: {:?}", book);
    Ok(())
}



#[cfg(not(feature = "mock"))]
pub async fn fetch_book(book_id: &str) -> Result<BookInfo, String> {
    let url = format!("/api/v1/books/{}", book_id);

    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch book {}: {}", book_id, resp.status()).into());
    }

    let book: BookInfo = resp.json().await.map_err(|e| e.to_string())?;
    Ok(book)
}

#[cfg(feature = "mock")]
pub async fn fetch_book(book_id: &str) -> Result<BookInfo, String> {
    let mock_book = match book_id {
        "b1" => BookInfo {
            id: "b1".to_string(),
            title: "Book One".to_string(),
            author_id: "a1".to_string(),
            series_id: "s1".to_string(),
            series_order: 1,
            file_path: "/mock/path/to/book1.epub".to_string(),
        },
        "b2" => BookInfo {
            id: "b2".to_string(),
            title: "Book Two".to_string(),
            author_id: "a1".to_string(),
            series_id: "s1".to_string(),
            series_order: 2,
            file_path: "/mock/path/to/book2.epub".to_string(),
        },
        _ => BookInfo {
            id: book_id.to_string(),
            title: format!("Mock Book {}", book_id),
            author_id: "a0".to_string(),
            series_id: "s0".to_string(),
            series_order: 0,
            file_path: "/mock/path/to/default.epub".to_string(),
        },
    };

    if mock_book.title.contains("Mock"){
        return Err("no book found".into());
    }

    Ok(mock_book)
}

#[cfg(not(feature = "mock"))]
pub async fn fetch_chapter_progress(
    book_id: &str,
) -> Result<ProgressResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "/api/v1/book/{}/chapterprogress",
        book_id
    );

    let resp = get_with_auth(&url).await?;
    if !resp.ok(){
        return Err(format!("Failed to fetch chapter progress {}: {}", book_id, resp.status()).into());

    }
    let chapter:ProgressResponse=resp.json().await.map_err(|e| e.to_string())?;

    Ok(chapter)
}

#[cfg(not(feature = "mock"))]
pub async fn fetch_book_progress(
    book_id: &str,
) -> Result<ProgressResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "/api/v1/book/{}/progress",
        book_id
    );

    let resp = get_with_auth(&url).await?;
    if !resp.ok(){
        return Err(format!("Failed to fetch book progress {}: {}", book_id, resp.status()).into());

    }
    let bookprogress:ProgressResponse=resp.json().await.map_err(|e| e.to_string())?;

    Ok(bookprogress)
}

#[cfg(feature = "mock")]
pub async fn fetch_chapter_progress(
    book_id: &str,
) -> Result<ProgressResponse, Box<dyn std::error::Error>> {
    let progress = match book_id {
        "b1" => ProgressResponse { progress: 0.25 },
        "b2" => ProgressResponse { progress: 0.75 },
        "b3" => ProgressResponse { progress: 1.0 },
        _ => ProgressResponse { progress: -1.0 },
    };

    if progress.progress < 0.0 {
        return Err("no chapter progress found".into());
    }

    Ok(progress)
}

#[cfg(feature = "mock")]
static MOCK_PROGRESS: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(0.1));

#[cfg(feature = "mock")]
pub async fn fetch_book_progress(
    book_id: &str,
) -> Result<ProgressResponse, Box<dyn std::error::Error>> {
    match book_id {
        "b1" => {
            let mut prog = MOCK_PROGRESS.lock().unwrap();

            // Increment by 10% on each call, max 1.0
            *prog = (*prog + 0.1).min(1.0);

            Ok(ProgressResponse { progress: *prog })
        }
        "b2" => Ok(ProgressResponse { progress: 0.9 }),
        "b3" => Ok(ProgressResponse { progress: 1.0 }),
        _ => Err("no book progress found".into()),
    }
}