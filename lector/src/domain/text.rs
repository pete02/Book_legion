use dioxus::{logger::tracing, prelude::*};
use once_cell::sync::Lazy;
use regex::Regex;


use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::Node;

use crate::domain;
use crate::infra;

static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());

static SENTENCE_SPLIT: Lazy<Regex>= Lazy::new(||Regex::new(r#"([^.!?…]+)[.!?…]+(\s*)"#).unwrap());



#[derive(Clone, PartialEq, Eq)]
pub struct TextHandler{
    pub book_id: String,
    pub chapter: Signal<String>,
    pub visible_text: Signal<String>,
    pub cur_text: Signal<String>,
    pub next_text: Signal<String>,
    pub chapter_idx: Signal<usize>,
    pub chapter_end: Signal<bool>
}

impl TextHandler {
    pub fn new(book_id: String)->TextHandler{
        return TextHandler { book_id:book_id,chapter:use_signal(||"".to_owned()), visible_text: use_signal(||"".to_owned()), next_text: use_signal(||"".to_owned()), cur_text: use_signal(||"".to_owned()), chapter_idx: use_signal(||0),chapter_end: use_signal(||false) }
    }
}

pub fn fetch_chapter(text_handler: &mut TextHandler){
    tracing::debug!("Fetch chapter: {}",(text_handler.chapter_idx)());
    let mut text_handler=text_handler.clone();
    spawn(async move{
        let html=infra::chapters::fetch_chapter(&text_handler.book_id, (text_handler.chapter_idx)()).await;    
        match html{
            Ok(txt)=>{
                let next=infra::chapters::fetch_cursor_text(&text_handler.book_id).await;
                if let Ok(text)=next{
                    text_handler.next_text.set(text.text.clone());
                    text_handler.cur_text.set(text.text);
                }
                text_handler.chapter.set(txt.text.clone());
                domain::page_forward::render_next_page(&mut text_handler);
            },
            Err(e)=>tracing::error!("error in getting chapter:{}",e)
        }
    });
}

pub fn use_text(book_id: String) -> TextHandler {
    let txt=TextHandler::new(book_id);
    let a=txt.clone();
    use_effect(move ||{
        let mut text=a.clone();
        fetch_chapter(&mut text);
    });
    return txt;
}

pub fn find_sentence_offset_with_html_backtrack(
    chapter_html: &str,
    start_snippet: &str,
) -> usize {
    let candidate_start = find_sentence_offset(chapter_html, start_snippet);

    let mut safe_start = candidate_start;

    while safe_start > 0 {
        if let Some(pos) = chapter_html[..safe_start].rfind('<') {
            if chapter_html[pos..].starts_with("</") {
                safe_start = pos;
            } else {
                safe_start = pos;
                break;
            }
        } else {
            safe_start = 0;
            break;
        }
    }


    safe_start
}


pub fn find_sentence_offset(chapter_html: &str, start_snippet: &str) -> usize {
    let snippet_sents = split_sentences(start_snippet);
    if snippet_sents.is_empty() {
        return 0;
    }

    let first_sent = &snippet_sents[0];
    let mut candidates = vec![];
    let mut search_start = 0;
    while let Some(pos) = chapter_html[search_start..].find(first_sent) {
        candidates.push((search_start + pos,first_sent.len()));
        search_start += pos + first_sent.len();
    }
    let org_length=candidates.len();

    while candidates.len() > 1 {
        candidates.retain(|&(start, _)| {
            let start=clamp_to_char_boundary(chapter_html, start);
            let normal=normalize_text(&chapter_html[start..]);
            if let Some(i)=normal.find(&normalize_text(start_snippet)){
                i<10
            }else{
                false
            }
        });
    }

    if candidates.len()==0{
        tracing::error!(" Searching for: {}",start_snippet);
        tracing::error!("Did not find any candidate. Search start: {}",search_start);
        tracing::error!("Tried to find: {}", first_sent);
        tracing::error!("Original length: {}", org_length);
        return 0;
    }
    candidates[0].0
}



fn split_sentences(text: &str) -> Vec<String> {

    SENTENCE_SPLIT.captures_iter(text)
        .map(|cap|{
            let s=cap.get(1).unwrap().as_str().trim();
            s.trim_matches(&['"', '“', '”', '\''][..]).to_string()
        })
        .filter(|s| s.chars().count() > 1) 
        .collect()
}

fn clamp_to_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    let mut i = idx;
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}
fn normalize_text(s: &str) -> String {

    strip_html(s).chars()
        .filter(|c| !matches!(c, '.' | '!' | '?' | '…' | '"' | '\'' | '“' | '”'))
        .collect::<String>()
        .trim()
        .to_lowercase()
}



pub fn strip_html(html: &str) -> String {
    HTML_TAG_RE.replace_all(html, "").to_string()
}



pub fn get_container()->HtmlElement{
    let document=web_sys::window().unwrap().document().unwrap();
    document
        .get_element_by_id("book-renderer").unwrap()
        .dyn_into::<HtmlElement>().unwrap()
}

pub fn set_text(child: &Node, text: String) ->String{
    let mut hid=text;
    let mut current = child.next_sibling();
    
    while hid.len() <= 50 {
        let node = match current {
            Some(ref n) => n,
            None => break,
        };

        if let Some(text) = node.text_content() {
            hid.push_str(&text);
        }

        current = node.next_sibling();
    }

    return hid.to_string();
}
