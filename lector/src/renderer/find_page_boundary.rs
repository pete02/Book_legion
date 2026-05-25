use std::{fmt::format, sync::Arc};
use dioxus::logger::tracing;
use web_sys::HtmlElement;

use crate::renderer::{self, sentence_boundaries};

#[derive(Clone)]
pub struct LayoutQuery{
    pub text: String,
    pub top: f64,
    pub bottom: f64,
    pub char_start: u32,
    pub children: Vec<LayoutQuery>,
    pub get_char_bottom: Arc<dyn Fn(u32) -> f64>,  // drop Send + Sync
    pub get_char_top: Arc<dyn Fn(u32) -> f64>,  

}




impl LayoutQuery {
    pub fn default() -> Self {
        Self {
            text: String::new(),
            top: 0.0,
            bottom: 0.0,
            char_start: 0,
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
        let my_end = self.char_start + self.text.len() as u32;
        let children_end = self.children.iter().map(|c| c.char_start+ c.text_len()).max().unwrap_or(0);
        my_end.max(children_end).saturating_sub(self.char_start)
    }


    pub fn print_text(&self)->String{
        let mut t=self.text.clone();
        for child in &self.children{
            t=format!("{}; child: {}",t, child.print_text());
        }
        t
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitResult {
    AllFit,
    NoneFit,
    LastFitting(u32),
}


impl std::fmt::Debug for LayoutQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutQuery")
            .field("text_len()", &self.text_len())
            .field("text", &self.text)
            .field("top", &self.top)
            .field("bottom", &self.bottom)
            .field("char_start", &self.char_start)
            .field("children", &self.children)
            .finish()
    }
}

pub fn last_fitting_char(layout: &LayoutQuery, page_height: f64) -> FitResult {
    
    console(&format!("last fitting char, has children: {:?}", layout.children.len()));
    console(&format!("layout {:?}; page height: {:?}", layout, page_height));
    console(&format!("layout text: {}", layout.print_text()));

    if layout.top > page_height {return FitResult::NoneFit;}
    if layout.bottom() == page_height {return FitResult::LastFitting(layout.char_start + layout.text_len() - 1);}
    if layout.bottom() < page_height {return FitResult::AllFit;}
    console("layout height ok");
    if layout.children.is_empty() && layout.text_len() == 0 {return FitResult::AllFit;} 
    if page_height <= 0.0 {return FitResult::NoneFit;}
    console("pre-checks cleared");
    let mut best_child_fit=FitResult::NoneFit;
    for child in &layout.children {
        console("going for child");
        let child_fit = last_fitting_char(child, page_height);
        console(&format!("returned: {:?}", child_fit));
        match child_fit{
            FitResult::AllFit=> best_child_fit = FitResult::LastFitting(child.char_start+child.text_len()),
            FitResult::NoneFit=> return best_child_fit,
            FitResult::LastFitting(_)=> {
                best_child_fit = child_fit
            },
        }
    }

    if layout.children.len() > 0{
        return best_child_fit;
    }

    let mut best_bottom=FitResult::NoneFit;
    if layout.children.len() == 0{
        for i in 0..layout.text_len() {
            let char_bottom = (layout.get_char_bottom)(i);
            console(&format!("checking: {}, got bottom: {}", i, char_bottom));
            if char_bottom <= page_height {
                best_bottom = FitResult::LastFitting(layout.char_start+i);
            } else {
                break;
            }
        }
    }

    console(&format!("Last fitting: {:?}, will return {:?}", layout, best_bottom));

   best_bottom
}


pub fn first_fitting_char(layout: &LayoutQuery, page_top: f64) -> FitResult {
    if layout.bottom() < page_top { return FitResult::NoneFit; }
    if layout.top == page_top   { return FitResult::LastFitting(layout.char_start); }
    if layout.top > page_top    { return FitResult::AllFit; }
    if layout.children.is_empty() && layout.text_len() == 0 { return FitResult::AllFit; }
    if page_top >= f64::MAX     { return FitResult::NoneFit; }

    // Walk children in reverse — last child is topmost in bottom→top flow
    for child in layout.children.iter().rev() {
        let child_fit = first_fitting_char(child, page_top);
        if child_fit != FitResult::AllFit && child_fit != FitResult::NoneFit {
            return child_fit;
        }
    }

    // Walk chars from last to first — char text_len()-1 is topmost
    let mut best_top = FitResult::NoneFit;
    for i in (0..layout.text_len()).rev() {
        let char_top = (layout.get_char_top)(i);
        if char_top >= page_top {
            best_top = FitResult::LastFitting(layout.char_start + i);
        } else {
            break;
        }
    }

    best_top
}

pub fn split_html_at(html: &str, byte_index: usize) -> &str {
    let safe_index = html
        .char_indices()
        .map(|(i, _)| i)
        .filter(|&i| i >= byte_index)
        .next()
        .unwrap_or(html.len());

    &html[safe_index..]
}


pub fn split_html_at_end(html: &str, byte_index: usize) -> &str {
    let safe_index = html
        .char_indices()
        .map(|(i, _)| i)
        .filter(|&i| i <= byte_index)
        .last()
        .unwrap_or(0);

    &html[..safe_index]
}

#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
fn console(text: &str){
    #[cfg(target_arch = "wasm32")]
    console::log_1(&JsValue::from_str(text));
}