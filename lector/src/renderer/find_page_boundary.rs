use std::{fmt::format, sync::Arc};
use dioxus::logger::tracing;
use web_sys::HtmlElement;

use crate::renderer::{self, sentence_boundaries};
use crate::renderer::layout_builder::LayoutQuery;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitResult {
    AllFit,
    NoneFit,
    LastFitting(u32),
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