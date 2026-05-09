# Save Cursor Mechanism in Text Renderer

This document explains how the save cursor functionality works in the text renderer component of the application.

## Overview

The save cursor mechanism is responsible for tracking the user's reading position in a book by saving the last viewed text snippet. This allows users to resume reading from where they left off.

## Key Components

### 1. Save Function (`save`)

The `save` function is called whenever the user's reading position changes. It's triggered by the `use_effect` hook in the `render` function.

```rust
fn save(chapter_html: String, book_id: String, index: usize, start: usize) {
    if start >= chapter_html.chars().count() {
        tracing::warn!("save skipped: start out of bounds ({})", start);
        return;
    }

    // Safely take a slice by chars
    let mut slice: String = chapter_html.chars().skip(start).take(1000).collect();

    let first_lt = slice.find('<');
    let first_gt = slice.find('>');

    if let Some(gt_pos) = first_gt {
        let should_trim = match first_lt {
            Some(lt_pos) => gt_pos < lt_pos,
            None => true,
        };

        if should_trim {
            slice = slice[gt_pos + 1..].to_string();
        }
    }


    if slice.len() > 50 {
        save_cursor(book_id, index, slice);
    }
}
```

**How it works:**
1. Takes a slice of 1000 characters from the current start position
2. Removes HTML tags from the beginning of the slice if needed
3. Only saves if the resulting text is longer than 50 characters
4. Calls `save_cursor` with the book ID, chapter index, and text snippet

### 2. Save Cursor Function (`save_cursor`)

```rust
pub fn save_cursor(book_id:String,index: usize, save_txt:String){
    spawn(async move{
        let cursor=infra::cursor::get_cursor_from_text(&book_id, index, &save_txt).await;
        match cursor {
            Err(e)=>tracing::error!("No cursor found: {}",e),
            Ok(c)=>{domain::cursor::save_bookcursor(c).await;}
        }
    });
}
```

**How it works:**
1. Uses `get_cursor_from_text` to generate a cursor based on the text snippet
2. If successful, saves the cursor using `save_bookcursor`

### 3. Cursor Generation (`get_cursor_from_text`)

```rust
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
```

**How it works:**
1. Sends the text snippet to the backend API endpoint
2. The backend processes the text to determine a cursor position
3. Returns a `BookCursor` object with chapter and chunk information

### 4. BookCursor Structure

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BookCursor {
    pub user_id: String,
    pub book_id: String,
    pub cursor: Cursor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub chapter: usize,
    pub chunk: usize,
}
```

**How it works:**
- `user_id`: Identifies which user's cursor this is
- `book_id`: Identifies which book the cursor belongs to
- `cursor.chapter`: The chapter number where the user left off
- `cursor.chunk`: The chunk within the chapter (determined by text position)

## Flow Diagram

```
User reads text → 
Position changes → 
save() function called → 
Text snippet extracted → 
get_cursor_from_text() → 
Backend API call → 
BookCursor generated → 
save_bookcursor() → 
Cursor saved to database
```

## Key Features

1. **Text Snippet Extraction**: Only saves meaningful text (at least 50 characters) to avoid saving empty or incomplete content
2. **HTML Tag Handling**: Removes HTML tags from the beginning of text snippets to ensure clean text
3. **Bounds Checking**: Ensures the save operation doesn't exceed text boundaries
4. **Asynchronous Operations**: All cursor operations are performed asynchronously to avoid blocking the UI
5. **Error Handling**: Proper error handling for cases where text is too short or API calls fail

## Backend Integration

The save cursor mechanism integrates with the backend through:
1. `/api/v1/books/{book_id}/chapters/{chapter_index}/cursor` endpoint
2. The backend processes the HTML snippet to determine appropriate chapter and chunk positions
3. Results are saved to the database using the `save_cursor` endpoint

This mechanism ensures that users can seamlessly resume reading from their last position across different sessions.