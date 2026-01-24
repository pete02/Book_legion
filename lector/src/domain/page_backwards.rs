use dioxus::{logger::tracing, prelude::*};


use crate::domain;
use crate::domain::text::TextHandler;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Document, HtmlElement, Node, Range, window};


pub fn render_prev_page(text_handler:&mut TextHandler){
    let mut offset=domain::text::find_sentence_offset_with_html_backtrack(&(text_handler.chapter)(), &(text_handler.cur_text)());
    text_handler.chapter_end.set(false);
    text_handler.next_text.set((text_handler.cur_text)());
    tracing::debug!("next: {}",(text_handler.next_text)());

    let container = domain::text::get_container();

    offset=adjust_for_open_tags(&(text_handler.chapter)(), offset);

    let vis=&(text_handler.chapter)()[..offset];
    tracing::debug!("vis: {}",vis);

    tracing::debug!("removed:{}",&(text_handler.chapter)()[offset..]);

    text_handler.visible_text.set(vis.to_owned());
    

    let mut handler_for_trim = text_handler.clone();
    let closure = Closure::once_into_js(move || {
        container.set_scroll_top(container.scroll_height());
        if let Some(node) = first_visible_text_node_recursive(&container.clone().into(), container.get_bounding_client_rect().top()) {
            let jump=domain::text::set_text(&node, node.text_content().unwrap_or_default());
            handler_for_trim.cur_text.set(jump);
            
        }
    });

    let window = web_sys::window().unwrap();
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            0,
        )
        .unwrap();

}


fn first_visible_text_node_recursive(node: &Node, container_top: f64) -> Option<Node> {
    if node.node_type() == Node::TEXT_NODE {
        let range = node.owner_document()?.create_range().ok()?;
        range.set_start(node, 0).ok()?;
        range.set_end(node, node.text_content()?.len() as u32).ok()?;
        let rect = range.get_bounding_client_rect();
        if rect.bottom() > container_top {
            return Some(node.clone());
        }
    } else {
        let children = node.child_nodes();
        for i in 0..children.length() {
            if let Some(text_node) = first_visible_text_node_recursive(&children.item(i)?, container_top) {
                return Some(text_node);
            }
        }
    }
    None
}


fn adjust_for_open_tags(chapter_html: &str, mut safe_start: usize) -> usize {
    let snippet = &chapter_html[..safe_start];
    
    // Find the last '<' before safe_start
    if let Some(pos) = snippet.rfind('<') {
        let tag_text = &snippet[pos..];
        tracing::debug!("tag: {}",tag_text)
        if tag_text.starts_with("</") {
            // closing tag → safe
            return safe_start;
        } else if tag_text.starts_with('<') {
            // opening tag → check if it is closed
            let tag_name = extract_tag_name(tag_text);
            if let Some(tag_name) = tag_name {
                // check if the tag is closed before safe_start
                let closing_tag = format!("</{}>", tag_name);
                if snippet.contains(&closing_tag) {
                    // tag closed already → safe
                    return safe_start;
                } else {
                    // tag not closed → move safe_start back to include the opening tag
                    return pos;
                }
            }
        }
    }

    safe_start
}

fn extract_tag_name(tag: &str) -> Option<String> {
    // matches <tag ...> or <tag>
    let tag = tag.trim_start_matches('<').trim_start_matches('/');
    let end = tag.find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(tag.len());
    if end == 0 { return None; }
    Some(tag[..end].to_string())
}