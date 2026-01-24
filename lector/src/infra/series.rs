use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::infra::auth::get_with_auth;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEntry {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub series_id: String,
    pub series_name: Option<String>,
    pub series_order: u32,
    pub file_path: String,
}

#[cfg(not(feature = "mock"))]
pub async fn fetch_series(series_id: &str) -> Result<Vec<BookEntry>, Box<dyn std::error::Error>> {

    let url = format!("/api/v1/series/{}", series_id);
    let resp = get_with_auth(&url).await?;
    
    if !resp.ok() {
        return Err(format!("series request failed: {}", resp.status()).into());
    }

    let books = resp.json::<Vec<BookEntry>>().await?;
    Ok(books)
}

#[cfg(feature = "mock")]
pub async fn fetch_series(series_id: &str) -> Result<Vec<BookEntry>, Box<dyn std::error::Error>> {
    let books = match series_id {
        "s1" => vec![
            BookEntry {
                id: "b1".into(),
                title: "Book One".into(),
                author_id: "a1".into(),
                series_id: "s1".into(),
                series_name: Some("Series One".into()),
                series_order: 1,
                file_path: "/path/to/book1.epub".into(),
            },
            BookEntry {
                id: "b2".into(),
                title: "Book Two".into(),
                author_id: "a1".into(),
                series_id: "s1".into(),
                series_name: Some("Series One".into()),
                series_order: 2,
                file_path: "/path/to/book2.epub".into(),
            },
        ],
        "s2" => vec![
            BookEntry {
                id: "b3".into(),
                title: "Book Three".into(),
                author_id: "a2".into(),
                series_id: "s2".into(),
                series_name: Some("Series Two".into()),
                series_order: 1,
                file_path: "/path/to/book3.epub".into(),
            },
            BookEntry {
                id: "b4".into(),
                title: "Book Four".into(),
                author_id: "a2".into(),
                series_id: "s2".into(),
                series_name: Some("Series Two".into()),
                series_order: 2,
                file_path: "/path/to/book4.epub".into(),
            },
        ],
        _ => vec![],
    };

    Ok(books)
}
