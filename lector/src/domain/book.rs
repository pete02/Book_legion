use dioxus::prelude::*;

use crate::{domain, infra::{self, book::BookInfo}};

#[derive(Clone, PartialEq, Eq)]
pub struct BookData{
    pub chapters: Vec<String>,
    pub current_chapter: usize,
    pub title: String,
    pub author: String,
    pub series_id: String,
}
fn error_book(err: String)->BookData{
    BookData{
        title: "error in getting book".to_string(),
        author: err,
        series_id: "".to_string(),
        chapters: vec![],
        current_chapter: 0,
    }
}


pub fn get_book_info(book_id: String)-> Signal<BookInfo>{
    let info=use_signal(|| BookInfo::new());

    return info;
}

pub fn update_book(info: BookInfo){
    info!("call for update");
}


pub async fn load_book(book_id: String,)->BookData{
    let mut chs=Vec::new();
    let cursor = domain::cursor::load_bookcursor(book_id.clone()).await;
    let chapters=infra::chapters::fetch_book_nav(&book_id).await;
    let book=infra::book::fetch_book(&book_id).await;

    match chapters{
        Err(_)=>{},
        Ok(vec)=>{
            for entry in vec{
                chs.push(entry.title);
            }
        }
    }
    let data=match book {
        Err(e)=>error_book(e),
        Ok(a)=>BookData { 
            chapters: chs, 
            current_chapter: cursor.cursor.chapter, 
            title: a.title, 
            author: a.author_id, 
            series_id: a.series_id, 
        }

    };

    return data;

}

pub fn use_book(book_id: String) -> Signal<BookData> {
    let book = use_signal(|| error_book("test".to_string()));
    use_effect(move ||{
        let mut book=book.clone();
        let book_id=book_id.clone();
        spawn(async move{
            book.set(load_book(book_id).await);
        });
    });
    book
}


pub fn select_chapter(book: Signal<BookData>, progress:Signal<f64>, index: usize, book_id: String) {
    let mut book=book.clone();
    let mut progress=progress.clone();
    book.with_mut(|f| f.current_chapter = index);

    // 2. Persist cursor asynchronously
    spawn(async move {
        let mut cursor = domain::cursor::load_bookcursor(book_id.clone()).await;
        cursor.cursor.chapter = index;
        cursor.cursor.chunk = 0;

        let _ = domain::cursor::save_bookcursor(cursor).await;
        match infra::book::fetch_book_progress(&book_id).await{
            Err(_)=>{},
            Ok(p)=>progress.set(p.progress),
        };
        
    });
}


pub async fn get_book_progress(book_id: String)->f64{
    match infra::book::fetch_book_progress(&book_id).await{
        Err(_)=>return 0.0,
        Ok(p)=>return p.progress
    }
}


pub fn get_chapter_progress(book_id: String)->Signal<f64>{
    let progress=use_signal(||0.0);
    use_effect(move || {
        let mut progress=progress.clone();
        let book_id=book_id.clone();
        spawn(async move{
            match infra::book::fetch_chapter_progress(&book_id).await{
                Err(_)=>progress.set(0.0),
                Ok(p)=>progress.set(p.progress)
            }
        });
    });
    return progress
}