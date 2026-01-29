use dioxus::{logger::tracing, prelude::*};


use crate::domain;
use crate::domain::text::TextHandler;

use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::Node;


pub fn render_prev_page(text_handler: &mut TextHandler) {
    tracing::info!("start to turn page");
    domain::text::save_cursor(text_handler, (text_handler.cur_text)());
    
    let offset = if (text_handler.start_at_end)() {
        let len = (text_handler.chapter)().len();
        text_handler.start_at_end.set(false);
        len
    } else if (text_handler.cur_text)().len() > 0 {
        let off = domain::text::find_sentence_offset_with_html_backtrack(
            &(text_handler.cur_text)(),
            &(text_handler.map)(),
        );
        text_handler.chapter_end.set(false);
        text_handler.next_text.set((text_handler.cur_text)());
        off
    } else {
        prev_chapter(text_handler);
        return;
    };

    let container = domain::text::get_container();
    let vis = &(text_handler.chapter)()[..offset];

    if !has_visible_text(vis) {
        prev_chapter(text_handler);
        return;
    }
    text_handler.next_text.set((text_handler.cur_text)());

    text_handler.set_visible(vis.to_owned());

    tracing::info!("height: {}", container.scroll_height());
    //tracing::info!("DOM: {}", container.inner_html());

    if let Some(node) = first_visible_text_container(&container.clone().into(), container.get_bounding_client_rect().top()) {
        tracing::debug!("First text: {:?}",node.text_content());
        let jump = domain::text::set_text(&node, node.text_content().unwrap_or_default());
        tracing::debug!("Set: {}", jump);
        text_handler.cur_text.set(jump);
    } else {
        tracing::warn!("offset: {}", offset);
        tracing::warn!("No visible text container, falling back to prev chapter");
        prev_chapter(text_handler);
    }
}



fn text_container_ancestor(mut node: Node) -> Option<web_sys::Element> {
    while let Some(parent) = node.parent_node() {
        if let Ok(el) = parent.clone().dyn_into::<web_sys::Element>() {
            // heuristic: block-ish or text-bearing
            let tag = el.tag_name();
            if matches!(
                tag.as_str(),
                "P" | "DIV" | "LI" | "BLOCKQUOTE" | "SECTION"
            ) {
                return Some(el);
            }
        }
        node = parent;
    }
    None
}

fn first_visible_text_container(
    node: &Node,
    container_top: f64,
) -> Option<web_sys::Element> {
    if node.node_type() == Node::TEXT_NODE {
        let doc = node.owner_document()?;
        let range = doc.create_range().ok()?;
        let text = node.text_content()?;
        if text.trim().is_empty() {
            return None;
        }

        range.set_start(node, 0).ok()?;
        range.set_end(node, text.len() as u32).ok()?;

        let rect = range.get_bounding_client_rect();
        if rect.bottom() > container_top {
            return text_container_ancestor(node.clone());
        }
    }

    let children = node.child_nodes();
    for i in 0..children.length() {
        if let Some(el) =first_visible_text_container(&children.item(i)?, container_top){
            return Some(el);
        }
    }
    None
}

fn prev_chapter(text_handler: &mut TextHandler){
    if (text_handler.chapter_idx)() > 0 {
        text_handler.chapter_idx.set((text_handler.chapter_idx)() - 1);
    }else{
        return;
    }
    text_handler.chapter_end.set(true);
    text_handler.start_at_end.set(true);
    domain::text::fetch_chapter(text_handler, render_prev_page);
}

pub fn has_visible_text(html: &str) -> bool {
    let mut inside_tag = false;

    for c in html.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag && !c.is_whitespace() => {
                return true;
            }
            _ => {}
        }
    }

    false
}