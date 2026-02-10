use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub chapter: usize,
    pub chunk: usize,
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cursor {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.chapter.cmp(&other.chapter) {
            Ordering::Equal => self.chunk.cmp(&other.chunk),
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
    pub fn new(user_id: &str, book_id: &str, chapter: usize, chunk: usize) -> BookCursor {
        BookCursor {
            user_id: user_id.to_owned(),
            book_id: book_id.to_owned(),
            cursor: Cursor { chapter, chunk },
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
        Err(_) => return  BookCursor::new(&username, &book_id, 0, 0),
    }
}

pub async fn save_bookcursor(cursor:BookCursor){
    let _=infra::save_cursor(&cursor).await;
}



use dioxus::logger::tracing;
use serde::{Deserialize, Serialize};
use crate::{domain, infra::cursor as infra};

