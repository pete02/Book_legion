use crate::domain::cursor::BookCursor;
use crate::infra::auth::{get_with_auth,post_with_auth};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CursorTextResponse {
    pub cursor: BookCursor,
    pub text: String, // 
}



#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CursorRequest {
    pub snippet_html: String,
}



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
use regex::Regex;
#[cfg(feature = "mock")]
use crate::domain::cursor::Cursor;
#[cfg(feature = "mock")]
use dioxus::logger::tracing;
#[cfg(feature = "mock")]
static MOCK_CURSOR: Lazy<Mutex<Option<BookCursor>>> =
    Lazy::new(|| Mutex::new(None));
#[cfg(feature = "mock")]
pub const MIN_START_TEXT_LEN: usize = 50;


#[cfg(feature = "mock")]
pub async fn fetch_cursor(book_id: &str) -> Result<BookCursor, String> {

    let mut guard = MOCK_CURSOR.lock().unwrap();

    let cursor = guard.get_or_insert_with(|| BookCursor {
        user_id: "mock_user".into(),
        book_id: book_id.to_string(),
        cursor: Cursor {
            chapter: 0,
            index: 0,
        },
    });

    Ok(cursor.clone())
}

#[cfg(feature = "mock")]
pub async fn save_cursor(cursor: &BookCursor) -> Result<(), String> {
    let mut guard = MOCK_CURSOR.lock().unwrap();
 //   tracing::debug!("save cursor: {:?}", cursor);
    *guard = Some(cursor.clone());
    Ok(())
}

#[cfg(not(feature = "mock"))]
pub async fn get_cursor_from_text(
    book_id: &str,
    chapter_index: usize,
    snippet_html: &str,
) -> Result<BookCursor, String> {

    if snippet_html.len() < 100{
        use dioxus::logger::tracing;
        tracing::error!("Snippet is too short: {}", snippet_html.len());
        return Err("snippet too short".into())
    }
    let payload = CursorRequest {
        snippet_html: snippet_html.to_string(),
    };

    let endpoint = format!(
        "/api/v1/books/{}/chapters/{}/cursor",
        book_id, chapter_index
    );

    let resp = post_with_auth(
        &endpoint,
        serde_json::to_string(&payload).map_err(|e| e.to_string())?,
    )
    .await?;
    let text=resp.text().await.map_err(|e|e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}


#[cfg(not(feature = "mock"))]
pub async fn fetch_cursor_text(book_id: &str) -> Result<CursorTextResponse, String> {
    let url = format!("/api/v1/cursors/{}/text", book_id);

    let resp = get_with_auth(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Failed to fetch cursor text for {}: {}", book_id, resp.status()).into());
    }

    let cursor_text: CursorTextResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(cursor_text)
}

#[cfg(feature = "mock")]
pub async fn fetch_cursor_text(book_id: &str) -> Result<CursorTextResponse, String> {
    use serde_json::json;
    use serde_json::Value;

    // Define the JSON data per book_id
    let json_data: Value = match book_id {
        "b1" => json!({
            "cursor": {
                "user_id": "pete",
                "book_id": "b1",
                "cursor": { "chapter": 0, "chunk": 0 }
            },
            "text": ""
        }),
        "b2" => json!({
            "cursor": {
                "user_id": "pete",
                "book_id": "b2",
                "cursor": { "chapter": 0, "chunk": 0 }
            },
            "text": "The passengers on the ship called her “Girl,” which was fine by her. After traveling with them for a month, she didn’t know their real names either. Her dark hair and golden skin stood out in this crowd like a crooked screw on a brand-new sheet of metal. The twenty-seven Tawny refugees on the ship all had milky complexions and hair as blue as the deepest ocean. That was what happened in the All Black."
        }),
        _ => json!({
            "cursor": {
                "user_id": "pete",
                "book_id": book_id,
                "cursor": { "chapter": 1, "chunk": 0 }
            },
            "text": "<p>Default cursor text.</p>"
        }),
    };

    // Deserialize JSON into your CursorTextResponse
    let cursor_text: CursorTextResponse = serde_json::from_value(json_data)
        .map_err(|e| format!("Failed to deserialize mock JSON: {}", e))?;

    Ok(cursor_text)
}


#[cfg(feature = "mock")]
pub async fn get_cursor_from_text(
    book_id: &str,
    chapter_index: usize,
    snippet_html: &str,
) -> Result<BookCursor, String> {
    use dioxus::logger::tracing;



    let visible_len = visible_text_len(snippet_html);

    if visible_len < 50 {
        return Err("Snippet must contain at least 50 visible characters".to_string());
    }


    // Deterministic chunk derivation
    let chunk = visible_len / 200;

    Ok(BookCursor {
        book_id: book_id.to_string(),
        user_id: "mock-user".to_string(),
        cursor: Cursor {
            chapter: chapter_index,
            index: chunk,
        },
    })
}

#[cfg(feature = "mock")]
fn visible_text_len(html: &str) -> usize {
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let text = tag_re.replace_all(html, "");
    text.trim().chars().count()
}
