use dioxus::logger::tracing;
use web_sys::{Element, Node};

use crate::domain::{self, text::TextHandler};

const DEBUG:bool=true;
#[derive(Debug)]
pub struct VisibleTextInfo {
    pub first_visible: String,
    pub last_visible: String,
    pub first_invisible_above: Option<String>,
    pub first_invisible_below: Option<String>,
    pub at_chapter_start: bool,
    pub at_chapter_end: bool,
}

pub fn get_visible_and_invisible_text(chapter_text: String, ahead: usize) -> VisibleTextInfo {
    let container = domain::text::get_container();
    

    let container_top = container.get_bounding_client_rect().top();
    let container_bottom = container.get_bounding_client_rect().bottom();

    fn all_text_nodes(node: &Node) -> Vec<Node> {
        let mut nodes = vec![];
        let children = node.child_nodes();
        for i in 0..children.length() {
            let child = children.get(i).unwrap();
            match child.node_type() {
                Node::TEXT_NODE => nodes.push(child.clone()),
                Node::ELEMENT_NODE => nodes.extend(all_text_nodes(&child)),
                _ => {}
            }
        }
        nodes
    }

    let text_nodes = all_text_nodes(&container.into());

    let mut first_visible: Option<String> = None;
    let mut last_visible: Option<String> = None;
    let mut first_invisible_above: Option<String> = None;
    let mut first_invisible_below: Option<String> = None;

    let mut found_visible = false;

    for node in &text_nodes {
        let elem: Option<Element> = node.parent_element();
        if let Some(el) = elem {
            let rect = el.get_bounding_client_rect();
            let text = node.text_content().unwrap_or_default();

            if rect.bottom() > container_top && rect.top() < container_bottom {
                if first_visible.is_none() {
                    first_visible = Some(text.clone());
                }
                last_visible = Some(text.clone());
                found_visible = true;
            } else if !found_visible {
                let snippet = text.chars().take(ahead).collect::<String>();
                first_invisible_above = Some(snippet);
            } else {
                if first_invisible_below.is_none() {
                    let snippet = text.chars().take(ahead).collect::<String>();
                    first_invisible_below = Some(snippet);
                }
            }
        }
    }

    // --- Correct chapter boundaries ---
    let at_chapter_start = if let Some(first) = &first_visible {
        chapter_text.starts_with(first)
    } else { false };

    let at_chapter_end = if let Some(last) = &last_visible {
        chapter_text.ends_with(last)
    } else { false };

    VisibleTextInfo {
        first_visible: first_visible.unwrap_or_default(),
        last_visible: last_visible.unwrap_or_default(),
        first_invisible_above,
        first_invisible_below,
        at_chapter_start,
        at_chapter_end,
    }
}