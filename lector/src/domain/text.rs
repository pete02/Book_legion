use dioxus::{logger::tracing, prelude::*};
use regex::Regex;

use crate::domain;
use crate::{infra, ui::Text};


#[derive(Clone, PartialEq, Eq)]
pub struct TextHandler{
    pub book_id: String,
    pub chapter: Signal<String>,
    pub visible_text: Signal<String>,
    pub start_text: Signal<String>,
    pub chapter_idx: Signal<usize>,
    pub chapter_end: Signal<bool>
}

impl TextHandler {
    pub fn New(book_id: String)->TextHandler{
        return TextHandler { book_id:book_id,chapter:use_signal(||"".to_owned()), visible_text: use_signal(||"".to_owned()), start_text: use_signal(||"".to_owned()), chapter_idx: use_signal(||0),chapter_end: use_signal(||false) }
    }
}

pub fn fetch_chapter(text_handler: &mut TextHandler){
    tracing::debug!("Fetch chapter: {}",(text_handler.chapter_idx)());
    let mut text_handler=text_handler.clone();
    spawn(async move{
        let html=infra::chapters::fetch_chapter(&text_handler.book_id, (text_handler.chapter_idx)()).await;    
        match html{
            Ok(txt)=>{
                text_handler.chapter.set(txt.text.clone());
                domain::page_forward::render_next_page(&mut text_handler);
            },
            Err(e)=>tracing::error!("error in getting chapter:{}",e)
        }
    });
}

pub fn use_text(book_id: String) -> TextHandler {
    let txt=TextHandler::New(book_id);
    let a=txt.clone();
    use_effect(move ||{
        let mut text=a.clone();
        fetch_chapter(&mut text);
    });
    return txt;
}
