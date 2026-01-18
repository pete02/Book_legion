use crate::domain::cursor::BookCursor;
use crate::infra::auth::{get_with_auth,post_with_auth};

#[cfg(not(feature = "mock"))]
pub async fn fetch_cursor(book_id: &str) -> Result<BookCursor, String> {
    let resp = get_with_auth(&format!("/api/v1/cursors/{book_id}")).await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!(
            "fetch cursor failed: {}",
            resp.status()
        ));
    }

    resp.json::<BookCursor>()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(not(feature = "mock"))]
pub async fn save_cursor(cursor: &BookCursor) -> Result<(), String> {
    let resp = post_with_auth("/api/v1/cursors/save", serde_json::to_string(cursor).map_err(|e| e.to_string())?).await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!(
            "save cursor failed: {}",
            resp.status()
        ))
    }
}

#[cfg(feature = "mock")]
use std::sync::Mutex;
#[cfg(feature = "mock")]
use once_cell::sync::Lazy;
#[cfg(feature = "mock")]
use crate::domain::cursor::Cursor;
#[cfg(feature = "mock")]
use dioxus::logger::tracing;
#[cfg(feature = "mock")]
static MOCK_CURSOR: Lazy<Mutex<Option<BookCursor>>> =
    Lazy::new(|| Mutex::new(None));

#[cfg(feature = "mock")]
pub async fn fetch_cursor(book_id: &str) -> Result<BookCursor, String> {

    let mut guard = MOCK_CURSOR.lock().unwrap();

    let cursor = guard.get_or_insert_with(|| BookCursor {
        user_id: "mock_user".into(),
        book_id: book_id.to_string(),
        cursor: Cursor {
            chapter: 0,
            chunk: 0,
        },
    });

    Ok(cursor.clone())
}

#[cfg(feature = "mock")]
pub async fn save_cursor(cursor: &BookCursor) -> Result<(), String> {
    let mut guard = MOCK_CURSOR.lock().unwrap();
    tracing::debug!("save cursor: {:?}", cursor);
    *guard = Some(cursor.clone());
    Ok(())
}