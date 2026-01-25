use std::collections::HashMap;
use std::fmt::format;

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
    pub html_offsets: HashMap<usize,usize>, // same length as plain.chars()
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
    pub map: Signal<TextMap>,
    pub start_at_end: Signal<bool>
}

impl TextHandler {
    pub fn new(book_id: String)->TextHandler{
        return TextHandler {start_at_end: use_signal(|| false),map: use_signal(||TextMap { plain: "".to_owned(), html_offsets: HashMap::new() }), book_id:book_id,chapter:use_signal(||"".to_owned()), visible_text: use_signal(||"".to_owned()), next_text: use_signal(||"".to_owned()), cur_text: use_signal(||"".to_owned()), chapter_idx: use_signal(||0),chapter_end: use_signal(||false) }
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
                text_handler.chapter.set(txt.text.clone());
                text_handler.map.set(build_text_map_from_html(&(text_handler.chapter)()));

                // Call the renderer you passed in
                renderer(&mut text_handler);
            }
            Err(e) => tracing::error!("error in getting chapter: {}", e),
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
            fetch_chapter(&mut text_handler, domain::page_forward::render_next_page);
        });
    });
    return txt;
}


pub fn find_sentence_offset_with_html_backtrack(
    chapter_html: &str,
    start_snippet: &str,
    map: &TextMap
) -> usize {
    let normalized = normalize_text(start_snippet);

    if let Some(pos) = map.plain.find(&normalized) {
        let end = (pos + 1000).min(map.plain.len());
        let val =map.html_offsets[&pos];
        let html_end = (val + 1000).min(chapter_html.len());

        val
    } else {
        tracing::debug!("no pos");
        0
    }
}


fn normalize_text(s: &str) -> String {

    HTML_TAG_RE.replace_all(s, "").to_string().chars()
        .filter(|c| !matches!(c, '.' | '!' | '?' | '…' | '"' | '\'' | '“' | '”'))
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


pub fn find_first_closing_tag(html: &str) -> Option<(usize, String)> {
    let mut rest = html;
    let mut offset = 0;

    while let Some(start) = rest.find("</") {
        let Some(end) = rest[start..].find('>') else { break; };
        let tag_text = &rest[start..start + end + 1];
        let tag_name = tag_text[2..tag_text.len() - 1].trim().to_string();
        let close_start = offset + start;
        let close_end = offset + start + end + 1;
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

        // skip normalized punctuation
        if matches!(c, '.' | '!' | '?' | '…' | '"' | '\'' | '“' | '”') {
            continue;
        }

        // add plain character
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
