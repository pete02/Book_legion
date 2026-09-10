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
    r#"/api/v1/books/{}/file\?file=([^"'&\s]+)"#,
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
    let captures= re.captures_iter(html);
    tracing::info!("found {} file urls", re.captures_iter(html).count());
    for cap in  captures{
        let full_url = cap.get(0).unwrap().as_str().to_string();
        let encoded_file = cap.get(1).unwrap().as_str().to_string();
        tracing::info!("found file url: {}", full_url);
        targets.entry(full_url).or_insert(encoded_file);
    }

    // Fetch all of them concurrently.
    let fetches = targets.into_iter().map(|(full_url, encoded_file)| async move {
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

use web_sys::{ Element, Node, NodeFilter, ShadowRoot};
pub fn resolve_offset_to_page(offset: i64, column_width_px: f64) -> Option<i32> {
    if !column_width_px.is_finite() || column_width_px <= 0.0 {
        return None;
    }

    let document = web_sys::window()?.document()?;
    let host = document.get_element_by_id("book-content")?;
    let shadow_root = host.shadow_root()?;

    let (node, local_offset) =
        find_node_at_offset(&document, &shadow_root, offset)?;

    let rect_left = caret_left(&document, &node, local_offset)?;

    let host_rect = host.get_bounding_client_rect();



    let local_x = rect_left - host_rect.left();
    web_sys::console::log_1(
        &format!(
            "offset={offset}, local_offset={local_offset}, \
             rect_left={rect_left}, host_left={}, \
             local_x={}, node_text={:?}",
            host_rect.left(),
            rect_left - host_rect.left(),
            node.text_content(),
        )
        .into(),
    );

    Some(((local_x / column_width_px).floor() as i32).max(0))
}
/// Walks all text nodes under `root` in document order, accumulating UTF-16
/// length, until the node containing `offset` is found.
///
/// Negative offsets clamp to the very first text node. Offsets past the end
/// of all text clamp to the end of the last text node (end of chapter).
fn find_node_at_offset(
    _document: &web_sys::Document,
    root: &web_sys::ShadowRoot,
    offset: i64,
) -> Option<(web_sys::Node, u32)> {
    let root: &web_sys::Node = root.unchecked_ref();

    let target = offset.max(0);
    let mut current = 0i64;
    let mut last_text_node = None;
    let mut last_text_len = 0u32;

    fn visit(
        node: &web_sys::Node,
        target: i64,
        current: &mut i64,
        last_text_node: &mut Option<web_sys::Node>,
        last_text_len: &mut u32,
    ) -> Option<(web_sys::Node, u32)> {
        if node.node_type() == web_sys::Node::TEXT_NODE {
            let text = node.text_content().unwrap_or_default();

            if text.trim().is_empty() {
                return None;
            }

            let len = text.encode_utf16().count() as i64;

            *last_text_node = Some(node.clone());
            *last_text_len = len as u32;

            if target < *current + len {
                return Some((
                    node.clone(),
                    (target - *current) as u32,
                ));
            }

            *current += len;
            return None;
        }

        let children = node.child_nodes();

        for i in 0..children.length() {
            if let Some(child) = children.item(i) {
                if let Some(result) = visit(
                    &child,
                    target,
                    current,
                    last_text_node,
                    last_text_len,
                ) {
                    return Some(result);
                }
            }
        }

        None
    }

    visit(
        root,
        target,
        &mut current,
        &mut last_text_node,
        &mut last_text_len,
    )
    .or_else(|| {
        last_text_node.map(|node| (node, last_text_len))
    })
}
/// Returns the viewport-relative x position of the caret at `local_offset`
/// within `node`'s text. Tries a non-collapsed range first (more reliably
/// gives a real rect in most engines); falls back to a collapsed range at
/// the end of the node if we're already at its last character.
fn caret_left(document: &Document, node: &Node, local_offset: u32) -> Option<f64> {
    let text_len = node
        .text_content()
        .unwrap_or_default()
        .encode_utf16()
        .count() as u32;

    let range = document.create_range().ok()?;
    range.set_start(node, local_offset).ok()?;

    let end_offset = (local_offset + 1).min(text_len);
    if end_offset > local_offset {
        range.set_end(node, end_offset).ok()?;
    }
    // else: local_offset == text_len == 0 case already filtered out by caller
    // (empty text nodes are skipped in find_node_at_offset), so this branch
    // only hits when local_offset == text_len for a non-empty node — a
    // collapsed range at the very end, which still yields a valid rect.

    let rect = range.get_bounding_client_rect();
    if rect.width() > 0.0 || rect.height() > 0.0 {
        return Some(rect.left());
    }

    // Fallback: some engines return an empty rect for boundary positions
    // (e.g. right at a line/column break). Try the other direction.
    if local_offset > 0 {
        let range2 = document.create_range().ok()?;
        range2.set_start(node, local_offset - 1).ok()?;
        range2.set_end(node, local_offset).ok()?;
        let rect2 = range2.get_bounding_client_rect();
        return Some(rect2.right());
    }

    Some(rect.left())
}