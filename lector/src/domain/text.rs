use dioxus::{logger::tracing, prelude::*};


use wasm_bindgen::JsCast;
use web_sys::Document;
use web_sys::HtmlElement;

use crate::infra;

pub fn fetch_and_apply_book_css(book_id: String, mut css_redy: Signal<bool>) {
    spawn(async move{
        match infra::chapters::fetch_book_css(&book_id).await {
            Ok(css_text) => {
                // Inject CSS into the document
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        let cleaned=strip_color_from_css(&css_text);
                        inject_css(&document, &book_id, &cleaned);
                        css_redy.set(true);
                        tracing::debug!("CSS loaded");
                    }
                }
            }
            Err(e) => tracing::error!("Failed to fetch book CSS: {}", e),
        }
    });
}
use regex::Regex;
pub fn strip_color_from_css(css: &str) -> String {
    let re = Regex::new(r"(?i)\b(background-)?color\s*:[^;]+;?\s*|\bfont-size\s*:[^;]+;?\s*").unwrap();
    re.replace_all(css, "").to_string()
}

fn inject_css(document: &Document, book_id: &str, css: &str) {
    let style_id = format!("book-css-{}", book_id);
    if let Some(existing) = document.get_element_by_id(&style_id) {
        existing.set_inner_html(css);
        return;
    }

    let style: HtmlElement = document
        .create_element("style")
        .unwrap()
        .dyn_into()
        .unwrap();
    style.set_id(&style_id);
    style.set_inner_html(css);

    if let Some(head) = document.head() {
        head.append_child(&style).unwrap();
    }
}