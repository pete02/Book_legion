use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node, Range, Text};
use crate::renderer::calculate_page_height::{LayoutQuery, split_html_at};

// ── Send+Sync wrapper ────────────────────────────────────────────────────────
// WASM is single-threaded; this is safe in practice.
struct WasmSend<T>(T);
unsafe impl<T> Send for WasmSend<T> {}
unsafe impl<T> Sync for WasmSend<T> {}



// ── Public entry point ───────────────────────────────────────────────────────

pub fn build_layout(element: &Element, char_start: u32,html: &str) -> LayoutQuery {
    console(&format!("build fo: {}",html));
    let mut layout = LayoutQuery::default();
    layout.char_start = char_start;

    let rect = element.get_bounding_client_rect();
    layout.top = rect.top();
    layout.bottom = rect.bottom();

    let child_nodes = element.child_nodes();

    if child_nodes.length() == 0 {
        layout.text = generate_element_html(element);
        attach_element_char_fns(&mut layout);
        return layout;
    }

    if child_nodes.length() == 1 {
        if let Some(node) = child_nodes.item(0) {
            if node.node_type() == Node::TEXT_NODE {
                layout.text = generate_element_html(element);
                if let Some(text_node) = node.dyn_ref::<Text>() {
                    attach_text_node_char_fns(&mut layout, text_node, char_start);
                } else {
                    attach_element_char_fns(&mut layout);
                }
                return layout;
            }
        }
    }

    // ── Track the running character offset across siblings ───────────────────
    let mut current_char = char_start;

    for i in 0..child_nodes.length() {
        let Some(node) = child_nodes.item(i) else { continue };

        if let Some(el) = node.dyn_ref::<Element>() {
            // Recurse — child_start is current_char, which advances after
            let child_outer = el.outer_html();
            let child_start = split_html_at(html, current_char as usize)
                .find(&child_outer)
                .map(|i| current_char as usize + i)
                .unwrap_or(current_char as usize);

            console(&format!("cur: {}", current_char));
            console(&format!("el: {:?}",el.outer_html()));
            let child = build_layout(el, child_start as u32, html);
            current_char += el.outer_html().len() as u32;  // <-- advance by what the child consumed
            layout.children.push(child);

        } else if let Some(text_node) = node.dyn_ref::<Text>() {
            let raw = text_node.node_value().unwrap_or_default();
            if raw.trim().is_empty() {
                continue;
            }
            let text_start = html[current_char as usize..]
                .find(&raw)
                .map(|i| current_char as usize + i)
                .unwrap_or(current_char as usize);


            let len = text_node.length();
            let mut child = LayoutQuery::default();
            child.text = raw;
            child.char_start = text_start as u32;  // <-- correct start for this text node
            attach_text_node_char_fns(&mut child, text_node, text_start as u32);

            current_char += len;  // <-- advance past this text node
            layout.children.push(child);
        }
    }

    attach_delegating_char_fns(&mut layout);

    layout
}
// ── Character-function attachment helpers ────────────────────────────────────

/// Leaf text node: use the Range API for exact per-character positions.
fn attach_text_node_char_fns(layout: &mut LayoutQuery, text_node: &Text, char_start: u32) {
    // Clone the JS object so each closure owns a handle
    let node_for_top  = WasmSend(text_node.clone().unchecked_into::<Node>());
    let node_for_bot  = WasmSend(text_node.clone().unchecked_into::<Node>());

    let text_len = layout.text.chars().count() as u32;

    layout.get_char_top = Arc::new(move |global_char: u32| {
        let offset = global_char.saturating_sub(char_start).min(text_len.saturating_sub(1));
        char_top(&node_for_top.0, offset)
    });

    layout.get_char_bottom = Arc::new(move |global_char: u32| {
        let offset = global_char.saturating_sub(char_start).min(text_len.saturating_sub(1));
        char_bottom(&node_for_bot.0, offset)
    });
}

/// Leaf element with no text: return the element's own bounding rect values.
fn attach_element_char_fns(layout: &mut LayoutQuery) {
    let top    = layout.top;
    let bottom = layout.bottom;
    layout.get_char_top    = Arc::new(move |_| top);
    layout.get_char_bottom = Arc::new(move |_| bottom);
}

/// Parent node: walk children to find the one that owns `global_char`.
fn attach_delegating_char_fns(layout: &mut LayoutQuery) {
    // Arc-wrap children so closures can share them without lifetime issues
    let children: Arc<Vec<LayoutQuery>> = Arc::new(std::mem::take(&mut layout.children));

    let children_top = Arc::clone(&children);
    let children_bot = Arc::clone(&children);

    // Put the children back (closures hold a shared ref via Arc)
    // We need a way to restore — store a clone; LayoutQuery should impl Clone.
    layout.children = (*children).clone(); // requires LayoutQuery: Clone

    layout.get_char_top = Arc::new(move |global_char: u32| {
        find_child_for_char(&children_top, global_char)
            .map(|c| (c.get_char_top)(global_char))
            .unwrap_or(0.0)
    });

    layout.get_char_bottom = Arc::new(move |global_char: u32| {
        find_child_for_char(&children_bot, global_char)
            .map(|c| (c.get_char_bottom)(global_char))
            .unwrap_or(0.0)
    });
}

fn find_child_for_char(children: &[LayoutQuery], global_char: u32) -> Option<&LayoutQuery> {
    children.iter().find(|c| {
        let end = c.char_start + c.text_len();
        global_char >= c.char_start && global_char < end
    })
}

// ── Range API helpers ────────────────────────────────────────────────────────

fn create_range() -> Option<Range> {
    web_sys::window()?.document()?.create_range().ok()
}

/// Top of the character at `local_offset` inside `node`.
fn char_top(node: &Node, local_offset: u32) -> f64 {
    let Some(range) = create_range() else { return 0.0 };
    // set_start / set_end take (node, char_offset)
    if range.set_start(node, local_offset).is_err()
        || range.set_end(node, local_offset + 1).is_err()
    {
        return 0.0;
    }
    range.get_bounding_client_rect().top()
}

/// Bottom of the character at `local_offset` inside `node`.
fn char_bottom(node: &Node, local_offset: u32) -> f64 {
    let Some(range) = create_range() else { return 0.0 };
    if range.set_start(node, local_offset).is_err()
        || range.set_end(node, local_offset + 1).is_err()
    {
        return 0.0;
    }
    range.get_bounding_client_rect().bottom()
}

// ── Misc helpers ─────────────────────────────────────────────────────────────

fn generate_element_html(element: &Element) -> String {
    element.outer_html()
}

#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
fn console(text: &str){
    #[cfg(target_arch = "wasm32")]
    //console::log_1(&JsValue::from_str(text));
    #[cfg(not(target_arch = "wasm32"))]
    println!("{}", text);
}