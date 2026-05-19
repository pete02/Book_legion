use std::sync::Arc;

// tests/dom_smoke_test.rs
use wasm_bindgen_test::*;
use web_sys::{window, HtmlElement,Element};
use wasm_bindgen::JsCast;
use crate::renderer::calculate_page_height::LayoutQuery;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn dom_element_has_nonzero_rect_when_attached() {
    let document = window().unwrap().document().unwrap();

    // 1. Create and style a span
    let span = document.create_element("span").unwrap();
    let span_html: &HtmlElement = span.dyn_ref().unwrap();
    
    // Use HtmlElement trait for style() method
    span_html.style().set_property("display", "inline-block").unwrap();
    span_html.style().set_property("font-size", "16px").unwrap();
    span_html.style().set_property("font-family", "monospace").unwrap();
    span.set_text_content(Some("hello"));

    // 2. Attach it — this is what gives it a real rect
    document.body().unwrap().append_child(&span).unwrap();

    // 3. Read back from DOM
    let rect = span.get_bounding_client_rect();

    // 4. Assert we got real dimensions
    assert!(rect.width() > 0.0,  "width should be > 0, got {}", rect.width());
    assert!(rect.height() > 0.0, "height should be > 0, got {}", rect.height());

    // Cleanup
    document.body().unwrap().remove_child(&span).unwrap();
}


use crate::renderer::layout_builder::build_layout;

#[wasm_bindgen_test]
fn build_layout_simple_text_element() {
    let document = window().unwrap().document().unwrap();
    
    let span = document.create_element("span").unwrap();
    span.set_text_content(Some("hello world"));
    document.body().unwrap().append_child(&span).unwrap();
    
    let layout = build_layout(&span);
    
    assert_eq!(layout.text, "<span>hello world</span>");
    assert_eq!(layout.text_len(), 24);
    assert!(layout.children.is_empty());
    
    document.body().unwrap().remove_child(&span).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_nested_elements() {
    let document = window().unwrap().document().unwrap();
    
    let parent = document.create_element("div").unwrap();
    let child = document.create_element("span").unwrap();
    child.set_text_content(Some("nested"));
    parent.append_child(&child).unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    
    let layout = build_layout(&parent);
    
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].text, "<span>nested</span>");
    
    document.body().unwrap().remove_child(&parent).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_text_and_children() {
    let document = window().unwrap().document().unwrap();
    
    let p = document.create_element("p").unwrap();
    p.set_text_content(Some("test"));
    
    let b = document.create_element("b").unwrap();
    b.set_text_content(Some("bold"));
    p.append_child(&b).unwrap();
    
    document.body().unwrap().append_child(&p).unwrap();
    
    let layout = build_layout(&p);
    
    // Parent has text "test" and one child
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 2);
    assert_eq!(layout.children[0].text, "test");
    assert_eq!(layout.children[1].text, "<b>bold</b>");
    
    document.body().unwrap().remove_child(&p).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_multiple_children() {
    let document = window().unwrap().document().unwrap();
    
    let div = document.create_element("div").unwrap();
    
    let span1 = document.create_element("span").unwrap();
    span1.set_text_content(Some("first"));
    div.append_child(&span1).unwrap();
    
    let span2 = document.create_element("span").unwrap();
    span2.set_text_content(Some("second"));
    div.append_child(&span2).unwrap();
    
    let span3 = document.create_element("span").unwrap();
    span3.set_text_content(Some("third"));
    div.append_child(&span3).unwrap();
    
    document.body().unwrap().append_child(&div).unwrap();
    
    let layout = build_layout(&div);
    
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 3);
    assert_eq!(layout.children[0].text, "<span>first</span>");
    assert_eq!(layout.children[1].text, "<span>second</span>");
    assert_eq!(layout.children[2].text, "<span>third</span>");
    
    document.body().unwrap().remove_child(&div).unwrap();
}

#[wasm_bindgen_test]
fn build_layout_deeply_nested() {
    let document = window().unwrap().document().unwrap();
    
    let outer = document.create_element("div").unwrap();
    let middle = document.create_element("div").unwrap();
    let inner = document.create_element("span").unwrap();
    
    inner.set_text_content(Some("deep"));
    middle.append_child(&inner).unwrap();
    outer.append_child(&middle).unwrap();
    document.body().unwrap().append_child(&outer).unwrap();
    
    let layout = build_layout(&outer);
    
    assert_eq!(layout.text, "");
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].text, "");
    assert_eq!(layout.children[0].children.len(), 1);
    assert_eq!(layout.children[0].children[0].text, "<span>deep</span>");
    
    document.body().unwrap().remove_child(&outer).unwrap();
}

// ... existing code ...