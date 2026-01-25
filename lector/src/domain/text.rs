use std::collections::HashMap;

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



#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TextMap {
    pub plain: String,
    pub html_offsets: Vec<usize>, // same length as plain.chars()
}

#[derive(Clone, PartialEq, Eq)]
pub struct TextHandler{
    pub book_id: String,
    pub chapter: Signal<String>,
    pub visible_text: Signal<String>,
    pub cur_text: Signal<String>,
    pub next_text: Signal<String>,
    pub chapter_idx: Signal<usize>,
    pub chapter_end: Signal<bool>,
    pub map: Signal<TextMap>
}

impl TextHandler {
    pub fn new(book_id: String)->TextHandler{
        return TextHandler {map: use_signal(||TextMap { plain: "".to_owned(), html_offsets: Vec::new() }), book_id:book_id,chapter:use_signal(||"".to_owned()), visible_text: use_signal(||"".to_owned()), next_text: use_signal(||"".to_owned()), cur_text: use_signal(||"".to_owned()), chapter_idx: use_signal(||0),chapter_end: use_signal(||false) }
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
                text_handler.map.set(build_text_map_from_html( &(text_handler.chapter)()));
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
        let mut text_handler=a.clone();
        
        spawn(async move{
            let next=infra::chapters::fetch_cursor_text(&text_handler.book_id).await;
            if let Ok(text)=next{
                text_handler.next_text.set(text.text.clone());
                text_handler.cur_text.set(text.text);
            }
            fetch_chapter(&mut text_handler);
        });
    });
    return txt;
}

pub fn find_sentence_offset_with_html_backtrack(
    chapter_html: &str,
    start_snippet: &str,
    map: &TextMap
) -> usize {
    tracing::debug!("searching: {}", start_snippet);
    if let Some(pos) = map.plain.find(&normalize_text(start_snippet)) {
        tracing::debug!("found pos: {}", pos);
        let val=map.html_offsets[pos];
        tracing::debug!("val: {}", val);
        val
    } else {
        tracing::debug!("no pos");
        0
    }
}



pub fn find_sentence_offset(
    start_snippet: &str,
    map: &TextMap
) -> usize {


    let plain_norm   = normalize_text(&map.plain);
    let snippet_norm = normalize_text(start_snippet);
    if let Some(pos) = plain_norm.find(&snippet_norm) {
        tracing::debug!("map: {:?}",map);
        return map.html_offsets[pos];
    }

    0
}


fn split_sentences(text: &str) -> Vec<String> {

    SENTENCE_SPLIT.captures_iter(text)
        .map(|cap|{
            let s=cap.get(1).unwrap().as_str().trim();
            s.trim_matches(&['"', '“', '”', '\'',' '][..]).to_string()
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

#[derive(Debug)]
enum HtmlToken<'a> {
    Open(&'a str),
    Close(&'a str),
    SelfClosing,
}

fn parse_tag(tag: &str) -> HtmlToken<'_> {
    if tag.starts_with("</") {
        HtmlToken::Close(tag[2..].trim_end_matches('>').trim())
    } else if tag.ends_with("/>") {
        HtmlToken::SelfClosing
    } else {
        HtmlToken::Open(
            tag.trim_start_matches('<')
               .trim_end_matches('>')
               .split_whitespace()
               .next()
               .unwrap()
        )
    }
}

fn collect_unbalanced_tags(html: &str) -> Vec<String> {
    let mut stack = Vec::new();

    let mut rest = html;
    while let Some(start) = rest.find('<') {
        let Some(end) = rest[start..].find('>') else { break };
        let tag = &rest[start..start + end + 1];

        match parse_tag(tag) {
            HtmlToken::Open(name) => stack.push(name.to_string()),
            HtmlToken::Close(name) => {
                if let Some(pos) = stack.iter().rposition(|t| t == name) {
                    stack.truncate(pos);
                }
            }
            HtmlToken::SelfClosing => {}
        }

        rest = &rest[start + end + 1..];
    }

    stack
}

fn repair_beginning(html: &str) -> String {
    let mut repaired = String::new();

    let unclosed = collect_unbalanced_tags(html);

    for tag in &unclosed {
        repaired.push_str(&format!("<{}>", tag));
    }

    repaired.push_str(html);
    repaired
}

fn repair_end(html: &str) -> String {
    let unclosed = collect_unbalanced_tags(html);
    let mut repaired = html.to_string();

    for tag in unclosed.iter().rev() {
        repaired.push_str(&format!("</{}>", tag));
    }

    repaired
}

pub fn normalize_html_fragment(html: &str) -> String {
    let repaired = repair_beginning(html);
    repair_end(&repaired)
}


pub fn build_text_map_from_html(chapter_html: &str) -> TextMap{
    let mut plain = String::new();
    let mut html_offsets = Vec::new();
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
            continue;
        }

        if matches!(c, '.' | '!' | '?' | '…' | '"' | '\'' | '“' | '”') {
            continue;
        }

        plain.push(c.to_ascii_lowercase());
        html_offsets.push(idx);
    }

    TextMap { plain, html_offsets }
}