use std::collections::HashMap;

use dioxus::{logger::tracing, prelude::*};


use wasm_bindgen::JsCast;
use web_sys::Document;
use web_sys::HtmlElement;

use crate::infra;


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
}



fn check_char_match(c:&char)->bool{
    matches!(c, '.'| ' ' | '!' | '?' | '…' | '"' | '\'' | '“' | '”' | ',')
}

fn normalize_char(c: &char) -> Option<char> {
    if check_char_match(c) {
        return None;
    }

    if c.is_ascii_alphanumeric() {
        return Some(c.to_ascii_lowercase());
    }


    None
}

pub fn normalize_text(s: &str) -> String {
    // Replace HTML entities if any
    let s = replace_html_entities(s);
    s.chars()
        .filter_map(|c| normalize_char(&c))  // keep only ASCII letters/digits
        .collect::<String>()
}

pub fn replace_html_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
     .replace("&amp;", "&")
     .replace("&#39;", "'")
     .replace("&quot;", "\"")
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
        tracing::error!("update worked");
        tracing::error!("tired to search for: {}",&normalized);
        tracing::error!("Originally: {}",start_snippet);
        tracing::error!("From: {}",map.plain);
        0
    }
}

pub fn build_text_map_from_html(chapter_html: &str) -> TextMap {
    let mut plain = String::new();
    let mut html_offsets = HashMap::new();
    let mut inside_tag = false;

    let mut idx_iter = chapter_html.char_indices().peekable();

    while let Some((idx, c)) = idx_iter.next() {
        // handle tags
        if c == '<' {
            inside_tag = true;
            continue;
        } else if c == '>' {
            inside_tag = false;
            continue;
        }
        if inside_tag { continue; }


        if let Some(char)=normalize_char(&c){
            plain.push(char);
            html_offsets.insert(plain.len() - 1, idx);
        }
    }

    TextMap { plain, html_offsets }
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