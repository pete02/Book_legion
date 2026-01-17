use crate::infra::auth::get_with_auth;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct BookInfo {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub series_id: String,
    pub series_order: usize,
    pub file_path: String,
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