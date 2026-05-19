use std::ops::Deref;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use crate::renderer::calculate_page_height::LayoutQuery;

pub fn build_layout(element: &Element) -> LayoutQuery {
    let mut layout = LayoutQuery::default();
    
    // Generate HTML markup for this element
    let html = generate_element_html(element);
    layout.text = html;
    
    // Get all child nodes (including text nodes)
    let child_nodes = element.child_nodes();
    
    for i in 0..child_nodes.length() {
        if let Some(node) = child_nodes.item(i) {
            // Try to cast to Element first
            if let Some(el) = node.dyn_ref::<Element>() {
                let child_layout = build_layout(&el);
                layout.children.push(child_layout);
            }
            // Handle text nodes separately
            else if let Some(text) = node.dyn_ref::<web_sys::Text>() {
                let text_content = text.node_value().unwrap_or_default();
                if !text_content.trim().is_empty() {
                    // Create a LayoutQuery for this text node
                    let mut text_layout = LayoutQuery::default();
                    text_layout.text = text_content;
                    layout.children.push(text_layout);
                }
            }
        }
    }
    
    layout
}

fn generate_element_html(element: &Element) -> String {
    let tag_name = element.tag_name().to_lowercase();
    let text_content = element.text_content().unwrap_or_default();
    
    if text_content.is_empty() {
        format!("<{}></{}>", tag_name, tag_name)
    } else {
        format!("<{}>{}</{}>", tag_name, text_content, tag_name)
    }
}