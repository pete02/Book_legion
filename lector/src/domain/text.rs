use std::collections::HashMap;

use dioxus::{logger::tracing, prelude::*};
use once_cell::sync::Lazy;
use regex::Regex;


use wasm_bindgen::JsCast;
use web_sys::Document;
use web_sys::HtmlElement;
use web_sys::Node;

use crate::domain;
use crate::domain::cursor::BookCursor;
use crate::domain::cursor::Cursor;
use crate::infra;

static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextMap {
    pub plain: String,
    pub html_offsets: HashMap<usize,usize>, // same length as plain.chars()
}

#[derive(Clone, PartialEq, Eq)]
pub struct TextHandler{
    pub book_id: String,
    pub chapter: Signal<String>,
    pub visible_text: Signal<String>,
    pub map: Signal<TextMap>,
    pub chapter_idx: Signal<usize>,
    pub cur_text: Signal<String>,
    pub next_text: Signal<String>,
    pub chapter_end: Signal<bool>,
    pub chapter_start: Signal<bool>,
    pub start_at_end: Signal<bool>,
    pub start_offset: Signal<usize>,
    pub end_offset: Signal<usize>
}

impl TextHandler {
    pub fn new(book_id: String)->TextHandler{
        return TextHandler {chapter_start: use_signal(||false),end_offset: use_signal(||0),start_offset: use_signal(||0), start_at_end: use_signal(|| false),map: use_signal(||TextMap { plain: "".to_owned(), html_offsets: HashMap::new() }), book_id:book_id,chapter:use_signal(||"".to_owned()), visible_text: use_signal(||"".to_owned()), next_text: use_signal(||"".to_owned()), cur_text: use_signal(||"".to_owned()), chapter_idx: use_signal(||0),chapter_end: use_signal(||false) }
    }

    pub fn set_visible(&mut self, visible_text:String){
        let vis=normalize_html_fragment(&visible_text);
        self.visible_text.set(vis);
    }
}

pub fn fetch_chapter<F>(text_handler: &mut TextHandler, renderer: F)
where
    F: FnOnce(&mut TextHandler) + Send + 'static,
{
    tracing::debug!("Fetch chapter: {}", (text_handler.chapter_idx)());
    let mut text_handler = text_handler.clone();
    
    spawn(async move {
        let html = infra::chapters::fetch_chapter(&text_handler.book_id, (text_handler.chapter_idx)()).await;

        match html {
            Ok(txt) => {
                text_handler.chapter.set(txt.clone());
                text_handler.map.set(build_text_map_from_html(&(text_handler.chapter)()));

                renderer(&mut text_handler);
            }
            Err(e) => tracing::error!("error in getting chapter: {}", e),
        }
    });
}



fn check_char_match(c:&char)->bool{
    matches!(c, '.'| ' ' | '!' | '?' | '…' | '"' | '\'' | '“' | '”' | ',')
}

fn normalize_text(s: &str) -> String {
    HTML_TAG_RE.replace_all(s, "").to_string().chars()
        .filter(|c| !check_char_match(c))
        .collect::<String>()
        .trim()
        .to_lowercase()
}

pub fn get_container()->HtmlElement{
    let document=web_sys::window().unwrap().document().unwrap();
    document
        .get_element_by_id("book-renderer").unwrap()
        .dyn_into::<HtmlElement>().unwrap()
}

pub fn find_first_closing_tag(html: &str) -> Option<(usize, String)> {
    let rest = html;
    let offset = 0;

    while let Some(start) = rest.find("</") {
        let Some(end) = rest[start..].find('>') else { break; };
        let tag_text = &rest[start..start + end + 1];
        let tag_name = tag_text[2..tag_text.len() - 1].trim().to_string();
        let close_start = offset + start;

        return Some((close_start, tag_name));
    }
    None
}

fn repair_beginning(html: &str) -> String {
    let first_end=find_first_closing_tag(html);
    if let Some((idx,tag))=first_end{
        let start=html.find(&format!("<{tag}>"));
        if let Some(sidx)=start && sidx<idx{
            return html.to_string();
        }else{
            if tag=="span"{
                return format!("<p><{tag}>{}",html);
            }else{
                return format!("<{tag}>{}",html);
            }
        }
    }else{
        return html.to_string();
    }

}
pub fn normalize_html_fragment(html: &str) -> String {
    let repaired = repair_beginning(html);
    repaired
}




pub fn find_sentence_offset_with_html_backtrack(
    start_snippet: &str,
    map: &TextMap
) -> usize {
    let normalized = normalize_text(start_snippet);

    if let Some(pos) = map.plain.find(&normalized) {
        let val =map.html_offsets[&pos];
        val
    } else {
        tracing::error!("no pos");
        tracing::error!("tired to search for: {}",normalize_text(start_snippet));
        tracing::error!("Originally: {}",start_snippet);
        tracing::error!("From: {}",map.plain);
        0
    }
}

pub fn build_text_map_from_html(chapter_html: &str) -> TextMap {
    let mut plain = String::new();
    let mut html_offsets = HashMap::new();
    let mut inside_tag = false;

    for (idx, c) in chapter_html.char_indices() {
        if c == '<' {
            inside_tag = true;
            continue;
        } else if c == '>' {
            inside_tag = false;
            continue;
        }

        if inside_tag {
            continue; // skip tag content
        }
        if check_char_match(&c) {
            continue;
        }

        plain.push(c.to_ascii_lowercase());
        html_offsets.insert(plain.len()-1,idx);
    }

    TextMap { plain, html_offsets }
}


pub fn save_cursor(text_handler: &mut TextHandler, save_txt:String){
    let text_handler=text_handler.clone();
    spawn(async move{
        if(text_handler.next_text)().len() > 0{
            let cursor=infra::cursor::get_cursor_from_text(&text_handler.book_id, (text_handler.chapter_idx)(), &save_txt).await;
            match cursor {
                Err(e)=>tracing::error!("No cursor founnd: {}",e),
                Ok(c)=>{domain::cursor::save_bookcursor(c).await;}
            }
        }else{
            let mut cursor=domain::cursor::load_bookcursor(text_handler.book_id).await;
            cursor.cursor.chapter=(text_handler.chapter_idx)();
            domain::cursor::save_bookcursor(cursor).await;
        }
    });
}

pub fn fetch_and_apply_book_css(book_id: String, mut css_redy: Signal<bool>) {
    spawn(async move{
        match infra::chapters::fetch_book_css(&book_id).await {
            Ok(css_text) => {
                // Inject CSS into the document
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        inject_css(&document, &book_id, &css_text);
                        css_redy.set(true);
                        tracing::debug!("CSS loaded");
                    }
                }
            }
            Err(e) => tracing::error!("Failed to fetch book CSS: {}", e),
        }
    });
}



fn inject_css(document: &Document, book_id: &str, css: &str) {
    let style_id = format!("book-css-{}", book_id);
    if let Some(existing) = document.get_element_by_id(&style_id) {
        existing.set_inner_html(css);
        return;
    }

    let style: HtmlElement = document
        .create_element("style")
        .unwrap()
        .dyn_into()
        .unwrap();
    style.set_id(&style_id);
    style.set_inner_html(css);

    if let Some(head) = document.head() {
        head.append_child(&style).unwrap();
    }
}