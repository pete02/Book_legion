use dioxus::{logger::tracing, prelude::*};
use web_sys::Node;


use crate::domain;
use crate::infra;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Document, HtmlElement, Range, window};
use crate::domain::text::TextHandler;

const DEBUG:bool=false;

#[macro_export]
macro_rules! debug_flagged {
    ($($arg:tt)*) => {
        {
            if DEBUG{
                tracing::debug!($($arg)*);
            }
            
        }
    };
}

pub fn render_next_page(text_handler: &mut TextHandler) {
    if (text_handler.chapter_end)() ==true{
        next_chapter(text_handler);
        return;
    }
    save_cursor(text_handler.clone());

    let chapter = (text_handler.chapter)();
    let start_text = (text_handler.next_text)();
    let start_offset = domain::text::find_sentence_offset_with_html_backtrack(&chapter, &start_text);

    let new_visible = chapter[start_offset..].to_string();
    text_handler.visible_text.set(new_visible);
    let container=domain::text::get_container();
    container.set_scroll_top(0);

    let mut handler_for_trim = text_handler.clone();
    let closure = Closure::once_into_js(move || {
        container.set_scroll_top(0);
        trim_overflowing_node(&mut handler_for_trim);
    });

    let window = window().unwrap();
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            0,
        )
        .unwrap();
}

fn save_cursor(text_handler: TextHandler){
    spawn(async move{
        if(text_handler.next_text)().len() > 0{
            let cursor=infra::cursor::get_cursor_from_text(&text_handler.book_id, (text_handler.chapter_idx)(), &(text_handler.next_text)()).await;
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

pub fn trim_overflowing_node(text_handler: &mut TextHandler){
    text_handler.cur_text.set((text_handler.next_text)());
    debug_flagged!("cur_text: {}", (text_handler.cur_text)());
    text_handler.next_text.set("".to_owned());
    let container=domain::text::get_container();
    debug_flagged!("running trim");
    let  Some(child)=first_overflowing_child(&container) else {
        debug_flagged!("No child found");
        text_handler.chapter_end.set(true);
        return;
    };
    let document=web_sys::window().unwrap().document().unwrap();

    if child.1{
        text_handler.next_text.set(domain::text::set_text(&child.0,child.0.text_content().unwrap_or_default()));
        debug_flagged!("next txt: {}",(text_handler.next_text)());
    }else{
        let (visible,hidden)=split_node_by_visible_words(
            &document,
            &child.0,
            container.get_bounding_client_rect().bottom()
        );

        let (vis,mut hid)=snap_to_last_sentence_break(&visible, &hidden);
        split_and_hide_node_in_chapter(&document, &child.0, &vis, &hid, text_handler);
        text_handler.next_text.set(domain::text::set_text(&child.0,hid));
        debug_flagged!("next txt: {}",(text_handler.next_text)());
        
    }
}


fn first_overflowing_child(
    container: &HtmlElement,
) -> Option<(Node,bool)> {
    let container_rect = container.get_bounding_client_rect();
    let children = container.child_nodes();

    for i in 0..children.length() {
        let child = children
            .item(i)?;

        let rect = child.clone().dyn_into::<HtmlElement>()
            .ok()?.get_bounding_client_rect();
        const EPSILON: f64 = 1.0;

        if rect.bottom() <= container_rect.bottom() + EPSILON {
            continue;
        }

        if container_rect.bottom() < rect.top()+EPSILON{
            return Some((child,true));
        }else{
            return Some((child,false));
        }
        
    }

    None
}


fn split_node_by_visible_words(
    document: &Document,
    child: &Node,
    container_bottom: f64,
)->(String,String){
    let node=child.clone().dyn_into::<HtmlElement>().unwrap();

    let full_text = node.inner_html();
    let mut visible_text = String::new();
    let mut hidden_text = String::new();
    let mut overflow_found = false;
    let words: Vec<&str> = full_text.split_whitespace().collect();
    let text_node = node
        .first_child()
        .expect("Node has no text child");
    let range: Range = document.create_range().unwrap();
    let mut current_offset = 0;

    for (_, word) in words.iter().enumerate() {
        if overflow_found {
            if !hidden_text.is_empty() {
                hidden_text.push(' ');
            }
            hidden_text.push_str(word);
            current_offset += word.len() + 1;
            continue;
        }

        let start_offset = current_offset;
        let end_offset = start_offset + word.len();

        if range.set_start(&text_node, start_offset as u32).is_err() ||
           range.set_end(&text_node, end_offset as u32).is_err() {
            overflow_found = true;
            hidden_text.push_str(word);
            continue;
        }

        let rect = range.get_bounding_client_rect();
        let bottom = rect.bottom();

        if bottom <= container_bottom + 1.0 {
            if !visible_text.is_empty() {
                visible_text.push(' ');
            }
            visible_text.push_str(word);
        } else {
            overflow_found = true;
            hidden_text.push_str(word);
        }

        current_offset = end_offset + 1;
    }

    return (visible_text, hidden_text);
}

pub fn snap_to_last_sentence_break(visible: &str, hidden: &str) -> (String, String) {
    let re = regex::Regex::new(r#"([.!?…]+["”']*\s*)"#).unwrap();
    let mut last_break_end = 0;

    for mat in re.find_iter(visible) {
        last_break_end = mat.end();
    }

    if last_break_end > 0 && last_break_end < visible.len() {
        let snapped_visible = visible[..last_break_end].to_string();

        let leftover = visible[last_break_end..].trim_start(); // remove any leading whitespace
        let snapped_hidden = format!("{} {}", leftover, hidden);

        (snapped_visible, snapped_hidden)
    } else {
        (visible.to_string(), hidden.to_string())
    }
}


pub fn split_and_hide_node_in_chapter(
    document: &Document,
    child: &Node,
    visible_html: &str,
    hidden_html: &str,
    text_handler: &mut TextHandler,
) -> Option<HtmlElement> {
    if hidden_html.is_empty() || visible_html.is_empty() {
        return None;
    }
     let node=child.clone().dyn_into::<HtmlElement>()
            .ok()?;

    let original_outer = node.outer_html();

    let visible_node = node.clone_node_with_deep(false).ok()?.dyn_into::<HtmlElement>().ok()?;
    visible_node.set_inner_html(visible_html);

    let hidden_node = document.create_element(&node.tag_name().to_lowercase()).ok()?;
    hidden_node.set_inner_html(hidden_html);
    
    if let Some(parent) = node.parent_node() {
        parent.insert_before(&visible_node, Some(&node)).ok()?;
        parent.insert_before(&hidden_node, Some(&node)).ok()?;
        parent.remove_child(&node).ok()?;
    }

    let chapter_html = (text_handler.chapter)();
    let new_outer_html = format!("{}{}", visible_node.outer_html(), hidden_node.outer_html());
    let updated_chapter = chapter_html.replacen(&original_outer, &new_outer_html, 1);
    text_handler.chapter.set(updated_chapter);
    Some(hidden_node.dyn_into::<HtmlElement>().ok()?)
}

fn next_chapter(text_handler: &mut TextHandler){
    text_handler.chapter_idx.set((text_handler.chapter_idx)()+1);
    text_handler.chapter_end.set(false);
    domain::text::fetch_chapter(text_handler);
}