use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub chapter: usize,
    #[serde(rename = "chunk")]
    pub index: usize,
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cursor {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.chapter.cmp(&other.chapter) {
            Ordering::Equal => self.index.cmp(&other.index),
            ord => ord,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BookCursor {
    pub user_id: String,
    pub book_id: String,
    pub cursor: Cursor,
}

impl PartialOrd for BookCursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BookCursor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cursor.cmp(&other.cursor)
    }
}

impl BookCursor {
    pub fn new( book_id: &str, chapter: usize, index: usize) -> BookCursor {
        let username=domain::login::current_name();
        if username ==""{
            tracing::error!("Could not get any username");
        }
        BookCursor {
            user_id: username,
            book_id: book_id.to_owned(),
            cursor: Cursor { chapter, index },
        }
    }
}

pub async fn load_bookcursor(book_id: String)->BookCursor{
    let username=domain::login::current_name();
    if username ==""{
        tracing::error!("Could not get any username");
    }
    match infra::fetch_cursor(&book_id).await{
        Ok(c) => return c,
        Err(_) => return  BookCursor::new(&book_id, 0, 0),
    }
}
use crate::infra::cursor::CursorTextResponse;




pub async fn save_cursor_text(book_id: &str, text: &str, chapter_idx:usize) -> Result<(), Box<dyn std::error::Error>> {
    match infra::get_cursor_from_text(book_id, chapter_idx, text).await {
        Ok(cursor) => {
            infra::save_cursor(&cursor).await?;
            Ok(())
        }
        Err(e) => {
            console(&format!("Error saving cursor text: {}", e));
            Err(e.into())
        }
    }
}

pub async fn save_bookcursor(cursor:BookCursor){
    let _=infra::save_cursor(&cursor).await;
}



use dioxus::logger::tracing;
use serde::{Deserialize, Serialize};
use crate::{domain, infra::cursor as infra};

#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
pub fn console(text: &str){
    #[cfg(target_arch = "wasm32")]
    console::log_1(&JsValue::from_str(text));
    #[cfg(not(target_arch = "wasm32"))]
    println!("{}", text);
}

