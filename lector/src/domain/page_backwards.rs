use dioxus::{logger::tracing, prelude::*};


use crate::domain::{self, text::normalize_html_fragment};
use crate::domain::text::TextHandler;
use crate::infra;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Document, HtmlElement, Node, Range, window};


pub fn render_prev_page(text_handler:&mut TextHandler){
    domain::text::save_cursor(text_handler, (text_handler.cur_text)());
    let offset;

    if (text_handler.start_at_end)(){
        offset=(text_handler.chapter)().len();
        text_handler.start_at_end.set(false);
    }else if (text_handler.cur_text)().len()>0{
        offset=domain::text::find_sentence_offset_with_html_backtrack(&(text_handler.chapter)(), &(text_handler.cur_text)(), &(text_handler.map)());
        text_handler.chapter_end.set(false);
        text_handler.next_text.set((text_handler.cur_text)());
    }else{
        prev_chapter(text_handler);
        return;
    }    

    

    let container = domain::text::get_container();
    let vis=&(text_handler.chapter)()[..offset];
    
    if !has_visible_text(vis) {
        prev_chapter(text_handler);
        return;
    }
        
    text_handler.set_visible(vis.to_owned());
    

    let mut handler_for_trim = text_handler.clone();
    let closure = Closure::once_into_js(move || {
        container.set_scroll_top(container.scroll_height());
        if let Some(node) = first_visible_text_container(&container.clone().into(), container.get_bounding_client_rect().top()) {
            let jump=domain::text::set_text(&node, node.text_content().unwrap_or_default());
            handler_for_trim.cur_text.set(jump);
        }else{
            tracing::debug!("offset: {}", offset);
            tracing::debug!("issues");
            prev_chapter(&mut handler_for_trim);
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
        if let Some(el) =
            first_visible_text_container(&children.item(i)?, container_top)
        {
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