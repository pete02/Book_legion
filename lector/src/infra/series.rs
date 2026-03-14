use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::infra::auth::{get_with_auth,delete_with_auth};
use crate::infra::auth::post_with_auth;



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


#[cfg(not(feature = "mock"))]
pub async fn delete_series(series_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("/api/v1/deleteseries/{}", series_id);
    let resp = delete_with_auth(&url).await?;
    
    if !resp.ok() {
        return Err(format!("series request failed: {}", resp.status()).into());
    }

    Ok(())
}


#[cfg(feature = "mock")]
pub async fn delete_series(series_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("deleted the series");
    Ok(())
}


#[cfg(not(feature = "mock"))]
pub async fn update_series_name(series_id: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {

    let url = format!("/api/v1/updateseries/{}", series_id);
    let body = serde_json::json!({ "name": name }).to_string();
    let resp = post_with_auth(&url, body).await?;

    if !resp.ok() {
        return Err(format!("update series request failed: {}", resp.status()).into());
    }

    Ok(())
}

#[cfg(feature = "mock")]
pub async fn update_series_name(series_id: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("updated series {} name to {}", series_id, name);
    Ok(())
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
        ],
        _ => vec![],
    };

    Ok(books)
}
