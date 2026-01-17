
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cursor {
    pub chapter: usize,
    pub chunk: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookCursor {
    pub user_id: String,
    pub book_id: String,
    pub cursor: Cursor,
}
impl BookCursor{
    pub fn new(user_id: String,book_id: String,chapter:usize, chunk: usize)->BookCursor{
        BookCursor{
            user_id: user_id,
            book_id: book_id,
            cursor: Cursor { chapter, chunk }
        }
    }
}


pub async fn load_bookcursor(book_id: String)->BookCursor{
    let username=domain::login::current_name();
    match infra::fetch_cursor(&book_id).await{
        Ok(c) => return c,
        Err(_) => return  BookCursor::new(username, book_id, 0, 0),
    }
}

pub async fn save_bookcursor(cursor:BookCursor){
    let _=infra::save_cursor(&cursor).await;
}



use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{domain, infra::cursor as infra};

pub fn use_cursor(book_id: String) -> Signal<Option<BookCursor>> {
    let mut cursor = use_signal(|| None);
    use_effect(move || {
        let book_id = book_id.clone();
        spawn(async move {
            cursor.set(Some(load_bookcursor(book_id).await));
        });
    });

    cursor
}

pub fn save_cursor(cursor: Signal<Option<BookCursor>>) {
    use_effect(move || {
        if let Some(c) = cursor() {
            spawn(async move {
                save_bookcursor(c);
            });
        }
    });
}