use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node, Range, Text};


// ── Send+Sync wrapper ────────────────────────────────────────────────────────
// WASM is single-threaded; this is safe in practice.
struct WasmSend<T>(T);
unsafe impl<T> Send for WasmSend<T> {}
unsafe impl<T> Sync for WasmSend<T> {}

#[derive(Clone)]
pub struct LayoutQuery{
    pub text: String,
    pub top: f64,
    pub bottom: f64,
    pub char_start: u32,
    pub end_tag_len: u32,
    pub children: Vec<LayoutQuery>,
    pub get_char_bottom: Arc<dyn Fn(u32) -> f64>,  // drop Send + Sync
    pub get_char_top: Arc<dyn Fn(u32) -> f64>,  

}

impl std::fmt::Debug for LayoutQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutQuery")
            .field("text_len()", &self.text_len())
            .field("text", &self.text)
            .field("top", &self.top)
            .field("bottom", &self.bottom)
            .field("char_start", &self.char_start)
            .field("end tag len", &self.end_tag_len)
            .field("children", &self.children)
            .finish()
    }
}
impl LayoutQuery {
    pub fn default() -> Self {
        Self {
            text: String::new(),
            top: 0.0,
            bottom: 0.0,
            char_start: 0,
            end_tag_len: 0,
            children: Vec::new(),
            get_char_bottom: Arc::new(|_| 0.0),
            get_char_top: Arc::new(|_| 0.0),
        }
    }
    pub fn bottom(&self) -> f64 {
        self.children
            .iter()
            .map(|c| c.bottom())
            .fold(self.bottom, f64::max)
    }
    pub fn text_len(&self) -> u32 {
        if self.children.len()==0{
            return self.text.len() as u32;
        }else{
            let children_end = self.children.iter().map(|c| c.char_start+ c.text_len()).max().unwrap_or(0);
            return children_end.saturating_sub(self.char_start)+self.end_tag_len
        }
    }


    pub fn print_text(&self)->String{
        let mut text=self.text.clone();
        for child in &self.children{
            text=format!("{}; child: {}",text, child.print_text());
        }
        text
    }

    pub fn find_leaf_text_for_char_index(&self, global_index: u32) -> Option<(&str, u32)> {
        if global_index >= self.char_start && global_index < self.char_start + self.text_len() {
            return Some((&self.text, global_index - self.char_start));
        }

        for child in &self.children {
            if let Some(found) = child.find_leaf_text_for_char_index(global_index) {
                return Some(found);
            }
        }

        None
    }

    pub fn subtree_end_char(&self) -> u32 {
        let mut end = self.char_start + self.text_len();
        for child in &self.children {
            end = end.max(child.subtree_end_char());
        }
        end
    }
}

// ── Public entry point ───────────────────────────────────────────────────────

pub fn build_layout_vec(viewport: &Element, html: &str) -> Vec<LayoutQuery> {
    let mut result = Vec::new();
    let mut current_char: u32 = 0;

    let child_nodes = viewport.child_nodes();
    for i in 0..child_nodes.length() {
        let Some(node) = child_nodes.item(i) else { continue };
        let Some(child) = build_child_node(&node, current_char, html) else { continue };

        current_char = child.char_start + child.text_len();
        result.push(child);
    }

    result
}

use web_sys::NodeList;

enum NodeKind {
    EmptyLeaf,
    TextLeaf(Text),
    Parent,
}

fn build_node(element: &Element, char_start: u32, html: &str) -> LayoutQuery {
    let child_nodes = element.child_nodes();

    match classify_node( &child_nodes) {
        NodeKind::EmptyLeaf        => build_empty_leaf(element, char_start),
        NodeKind::TextLeaf(text)   => build_text_leaf(element, char_start, &text),
        NodeKind::Parent           => build_parent(element, char_start, html),
    }
}

fn classify_node(child_nodes: &NodeList) -> NodeKind {
    if child_nodes.length() == 0 {
        return NodeKind::EmptyLeaf;
    }
    if child_nodes.length() > 1{
        return  NodeKind::Parent;
    }

    let single_text_node= child_nodes.item(0)
            .filter(|n| n.node_type() == Node::TEXT_NODE)
            .and_then(|n| n.dyn_ref::<Text>().map(|t| t.clone()))
            .map(NodeKind::TextLeaf);

    single_text_node.unwrap_or(NodeKind::Parent)
}


fn build_empty_leaf(element: &Element, char_start: u32) -> LayoutQuery {
    let mut layout = base_layout(element, char_start);
    layout.text = element.outer_html();
    attach_element_char_functions(&mut layout);
    layout
}

fn build_text_leaf(element: &Element, char_start: u32, text_node: &Text) -> LayoutQuery {
    let mut layout = base_layout(element, char_start);
    layout.text = element.outer_html();
    attach_text_node_char_functions(&mut layout, text_node, char_start);
    layout
}

fn build_parent(element: &Element, char_start: u32, html: &str) -> LayoutQuery {
    let mut layout = base_layout(element, char_start);
    let mut current_char = inner_content_start(element, char_start);

    let child_nodes = element.child_nodes();
    for i in 0..child_nodes.length() {
        let Some(node) = child_nodes.item(i) else { continue };
        let Some(child) = build_child_node(&node, current_char, html) else { continue };

        current_char = child.char_start + child.text.len() as u32;
        layout.children.push(child);
    }

    attach_delegating_char_functions(&mut layout);
    layout
}

fn build_child_node(node: &Node, current_char: u32, html: &str) -> Option<LayoutQuery> {
    if let Some(el) = node.dyn_ref::<Element>() {
        build_element_child(el, current_char, html)
    } else if let Some(text_node) = node.dyn_ref::<Text>() {
        build_text_child(text_node, current_char, html)
    } else {
        None
    }
}

fn base_layout(element: &Element, char_start: u32) -> LayoutQuery {
    let mut layout = LayoutQuery::default();
    let rect = element.get_bounding_client_rect();
    let outer = element.outer_html();
    let inner = element.inner_html();
    let end_tag_len=outer.len() - inner.len() - (outer.find(&inner).unwrap_or(0));

    layout.char_start = char_start;
    layout.end_tag_len =  end_tag_len as u32;
    layout.top = rect.top();
    layout.bottom = rect.bottom();
    layout
}

fn inner_content_start(element: &Element, char_start: u32) -> u32 {
    let outer = element.outer_html();
    let inner = element.inner_html();
    let tag_len = outer.find(&inner).unwrap_or(0);
    char_start + tag_len as u32
}

fn build_element_child(el: &Element, current_char: u32, html: &str) -> Option<LayoutQuery> {
    let outer = el.outer_html();
    let child_start = html[current_char as usize..]
        .find(&outer)
        .map(|offset| current_char + offset as u32)
        .unwrap_or(current_char);
    Some(build_node(el, child_start, html))
}

fn build_text_child(text_node: &Text, current_char: u32, html: &str) -> Option<LayoutQuery> {
    let raw = text_node.node_value().unwrap_or_default();
    if raw.trim().is_empty() {
        return None;
    }
    let text_start = html[current_char as usize..]
        .find(&raw)
        .map(|offset| current_char + offset as u32)
        .unwrap_or(current_char);

    let mut child = LayoutQuery::default();
    child.text = raw;
    child.char_start = text_start;
    attach_text_node_char_functions(&mut child, text_node, text_start);
    Some(child)
}


fn attach_text_node_char_functions(layout: &mut LayoutQuery, text_node: &Text, char_start: u32) {
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

fn attach_element_char_functions(layout: &mut LayoutQuery) {
    let top    = layout.top;
    let bottom = layout.bottom;
    layout.get_char_top    = Arc::new(move |_| top);
    layout.get_char_bottom = Arc::new(move |_| bottom);
}

fn attach_delegating_char_functions(layout: &mut LayoutQuery) {
    let children: Arc<Vec<LayoutQuery>> = Arc::new(std::mem::take(&mut layout.children));
    let children_top = Arc::clone(&children);
    let children_bot = Arc::clone(&children);
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

fn create_range() -> Option<Range> {
    web_sys::window()?.document()?.create_range().ok()
}

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