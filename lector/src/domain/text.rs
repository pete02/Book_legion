use dioxus::{logger::tracing, prelude::*};


use wasm_bindgen::JsCast;
use web_sys::Document;
use web_sys::HtmlElement;

use crate::domain;
use crate::domain::cursor::BookCursor;
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


use dioxus::document;

/// JS executed to find the raw HTML offset corresponding to the first
/// visible character in the paginated viewport.
///
/// Approach: hit-test the top-left corner of the visible `#book-renderer`
/// window (not `#book-content`, which may be translated out of view), find
/// the DOM text node/offset under that point, then measure how much
/// serialized HTML precedes that point within `#book-content`. The
/// `#page-nav-overlay` buttons are temporarily made transparent to
/// hit-testing so they don't shadow the text underneath.
const PAGE_OFFSET_SCRIPT: &str = r#"
    const viewport = document.getElementById('book-renderer');
    const content = document.getElementById('book-content');
    const overlay = document.getElementById('page-nav-overlay');
    if (!viewport || !content) { return -1; }

    const rect = viewport.getBoundingClientRect();
    const x = rect.left + 2;
    const y = rect.top + 2;

    let prevPointerEvents = null;
    if (overlay) {
        prevPointerEvents = overlay.style.pointerEvents;
        overlay.style.pointerEvents = 'none';
    }

    let node, offset;
    try {
        if (document.caretRangeFromPoint) {
            const range = document.caretRangeFromPoint(x, y);
            if (!range) { return -1; }
            node = range.startContainer;
            offset = range.startOffset;
        } else if (document.caretPositionFromPoint) {
            const pos = document.caretPositionFromPoint(x, y);
            if (!pos) { return -1; }
            node = pos.offsetNode;
            offset = pos.offset;
        } else {
            return -1;
        }
    } finally {
        if (overlay) {
            overlay.style.pointerEvents = prevPointerEvents;
        }
    }

    const full = document.createRange();
    full.setStart(content, 0);
    full.setEnd(node, offset);

    const frag = full.cloneContents();
    const wrapper = document.createElement('div');
    wrapper.appendChild(frag);
    return wrapper.innerHTML.length;
"#;

pub fn measure_element_width(element_id: &str) -> Option<f64> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document.get_element_by_id(element_id)?;
    let el: HtmlElement = el.dyn_into().ok()?;
    let width = el.client_width() as f64;
    if width > 0.0 { Some(width) } else { None }
}

pub fn compute_total_pages(scroll_width: f64, client_width: f64) -> Option<i32> {
    if client_width <= 0.0 {
        return None;
    }
    let pages = (scroll_width / client_width).ceil() as i32 -1;
    Some(pages.max(1))
}

pub fn measure_total_pages(element_id: &str) -> Option<i32> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document.get_element_by_id(element_id)?;
    let el: HtmlElement = el.dyn_into().ok()?;
    compute_total_pages(el.scroll_width() as f64, el.client_width() as f64)
}

pub async fn measure_current_page_html_offset() -> Option<i64> {
    match document::eval(PAGE_OFFSET_SCRIPT).await {
        Ok(value) => {
            let index = value.as_i64().unwrap_or(-1);
            if index >= 0 { Some(index) } else { None }
        }
        Err(err) => {
            dioxus::logger::tracing::warn!("failed to compute page offset: {err:?}");
            None
        }
    }
}


pub async fn save_cursor(page: i32, index: i64, chapter_idx: usize, book_id: &str){
    dioxus::logger::tracing::info!(
        "page {page} -> html offset at top of page: {index}"
    );
    let cursor=BookCursor::new(book_id, chapter_idx, index as usize);
    domain::cursor::save_bookcursor(cursor).await;
}

pub async fn get_new_chapter(chapter_idx: usize, book_id: &str, mut chapter_signal: Signal<Option<String>>) {
    match infra::chapters::fetch_chapter(book_id, chapter_idx).await{
        Ok(text)=>{
            chapter_signal.set(Some(text))
        },
        Err(err)=>{
            dioxus::logger::tracing::error!("failed to fetch chapter: {err:?}");
            chapter_signal.set(Some(format!("<div><p>got error in requesting chapter {:?}, Err: {:?}</p></div>",chapter_idx, err)));
        }
    }
}

pub async fn find_page_for_offset(
    target_offset: i64,
    total_pages: i32,
    mut set_page: impl FnMut(i32),
) -> i32 {
    let (mut lo, mut hi) = (0, total_pages - 1);
    let mut best = 0;

    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        set_page(mid);
        gloo_timers::future::TimeoutFuture::new(0).await;

        match measure_current_page_html_offset().await {
            Some(offset) if offset <= target_offset => {
                best = mid;
                lo = mid + 1;
            }
            _ => {
                hi = mid - 1;
            }
        }
    }

    best
}