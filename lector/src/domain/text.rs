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
                        //let cleaned=strip_color_from_css(&css_text);
                        //inject_css(&document, &book_id, &cleaned);
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
    const hostEl = document.getElementById('book-content');
    if (!viewport || !hostEl) { return -1; }

    const shadow = hostEl.shadowRoot;
    if (!shadow) { return -1; }

    const vRect = viewport.getBoundingClientRect();

    function intersectsViewport(r) {
        return r.width > 0 && r.height > 0
            && r.bottom > vRect.top && r.top < vRect.bottom
            && r.right > vRect.left && r.left < vRect.right;
    }

    const walker = document.createTreeWalker(shadow, NodeFilter.SHOW_TEXT);
    const range = document.createRange();
    let found = null;

    let node;
    while ((node = walker.nextNode())) {
        const text = node.textContent;
        if (!text || !text.trim().length) continue;

        range.selectNodeContents(node);
        const rects = range.getClientRects();
        let coarseHit = false;
        for (const r of rects) {
            if (intersectsViewport(r)) { coarseHit = true; break; }
        }
        if (!coarseHit) continue;

        for (let i = 0; i < text.length; i++) {
            range.setStart(node, i);
            range.setEnd(node, i + 1);
            const r = range.getBoundingClientRect();
            if (intersectsViewport(r)) {
                found = { node, offset: i };
                break;
            }
        }
        if (!found) found = { node, offset: text.length };
        break;
    }

    if (!found) { return -1; }

    const full = document.createRange();
    full.setStart(shadow, 0);
    full.setEnd(found.node, found.offset);
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

pub async fn get_new_chapter(
    chapter_idx: usize,
    book_id: &str,
    mut chapter_signal: Signal<Option<String>>,
) {
    match infra::chapters::fetch_chapter(book_id, chapter_idx).await {
        Ok(text) => {
            let with_blobs = inline_file_urls_as_blobs(book_id, &text).await;
            chapter_signal.set(Some(with_blobs));
        }
        Err(err) => {
            dioxus::logger::tracing::error!("failed to fetch chapter: {err:?}");
            chapter_signal.set(Some(format!(
                "<div><p>got error in requesting chapter {:?}, Err: {:?}</p></div>",
                chapter_idx, err
            )));
        }
    }
}
use std::collections::HashMap;

async fn inline_file_urls_as_blobs(book_id: &str, html: &str) -> String {
    // Matches the exact shape the Go backend emits, including the stray `&`
    // before the first query param (`?&file=`) as well as the plain `?file=` form.
    let pattern = format!(
        r#"/api/v1/books/{}/file\?&?file=([^"'&\s]+)"#,
        regex::escape(book_id)
    );

    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            dioxus::logger::tracing::error!("bad file-url regex: {e:?}");
            return html.to_string();
        }
    };

    // Dedupe by the *full matched URL* so we only fetch each asset once,
    // even if it's referenced multiple times in the chapter (e.g. shared CSS).
    let mut targets: HashMap<String, String> = HashMap::new(); // full_url -> encoded_file_path
    for cap in re.captures_iter(html) {
        let full_url = cap.get(0).unwrap().as_str().to_string();
        let encoded_file = cap.get(1).unwrap().as_str().to_string();
        targets.entry(full_url).or_insert(encoded_file);
    }

    // Fetch all of them concurrently.
    let fetches = targets.into_iter().map(|(full_url, encoded_file)| async move {
        // encoded_file is already percent-encoded exactly as the backend produced it,
        // so pass it straight through rather than re-encoding.
        let result = infra::book::fetch_book_file_as_blob_url(book_id, &encoded_file).await;
        (full_url, result)
    });

    let results = futures::future::join_all(fetches).await;

    let mut out = html.to_string();
    for (full_url, result) in results {
        match result {
            Ok(blob_url) => {
                out = out.replace(&full_url, &blob_url);
            }
            Err(err) => {
                dioxus::logger::tracing::error!(
                    "failed to fetch file for blob url ({full_url}): {err:?}"
                );
                // leave the original URL in place on failure so the browser
                // at least attempts the direct (auth-less) request as a fallback
            }
        }
    }

    out
}

pub async fn find_page_for_offset(
    target_offset: i64,
    total_pages: i32,
) -> i32 {
    let (mut lo, mut hi) = (0, total_pages - 1);
    let mut best = 0;

    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        tracing::debug!("testing page {mid}");
        gloo_timers::future::TimeoutFuture::new(0).await;

        match measure_current_page_html_offset().await {
            Some(offset) if offset <= target_offset => {
                tracing::debug!("Got offset {offset} <= {target_offset}");
                best = mid;
                lo = mid + 1;
            }
            _ => {
                hi = mid - 1;
                tracing::debug!("no offset found for page {mid}");
            }
        }
    }

    best
}