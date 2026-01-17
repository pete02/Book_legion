#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Chapter {
    pub index: usize,
    pub number: usize,
    pub title: String,
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PrettySpineItem {
    #[serde(rename = "Index")]
    pub index: usize,
    #[serde(rename = "Number")]
    pub number: usize,
    #[serde(rename = "Title")]
    pub title: String,
}

use crate::infra::auth::get_with_auth;
#[cfg(not(feature = "mock"))]
pub async fn fetch_book_nav(book_id: &str) -> Result<Vec<PrettySpineItem>, String> {
    let url = format!("/api/v1/books/{}/nav", book_id);

    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch chapters for {}: {}", book_id, resp.status()).into());
    }

    let chapters: Vec<PrettySpineItem> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(chapters)
}


#[cfg(feature = "mock")]
pub async fn fetch_book_nav(book_id: &str) -> Result<Vec<PrettySpineItem>, String> {
    let chapters = match book_id {
        "b1" => vec![
            PrettySpineItem { index: 0, number: 1, title: "Chapter 1".to_string() },
            PrettySpineItem { index: 1, number: 2, title: "Chapter 2".to_string() },
        ],
        "b2" => vec![
            PrettySpineItem { index: 0, number: 1, title: "First Chapter".to_string() },
            PrettySpineItem { index: 1, number: 2, title: "Second Chapter".to_string() },
        ],
        _ => vec![
            PrettySpineItem { index: 0, number: 1, title: "Default Chapter".to_string() },
        ],
    };

    Ok(chapters)
}
