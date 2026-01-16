use dioxus::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct BookData{
    pub chapters: Vec<String>,
    pub current_chapter: usize,
    pub title: String,
    pub author: String,
    pub series_id: String,
    pub cover: String
}


pub fn load_book(book_id: String)->Signal<BookData>{
    use_signal(||BookData { 
        chapters: vec!["test".to_owned(),"test2".to_owned()], 
        title: "test".to_owned(), author: "test_author".to_owned(), 
        series_id: "s1".to_owned(), 
        cover: format!("/api/v1/books/{}/cover",book_id),
        current_chapter:1 
    })
}

pub fn select_chapter(mut book: Signal<BookData>, index: usize){
    book.with_mut(|f|f.current_chapter=index)
}