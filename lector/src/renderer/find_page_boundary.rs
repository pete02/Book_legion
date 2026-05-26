use crate::renderer::layout_builder::LayoutQuery;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitResult {
    AllFit,
    NoneFit,
    LastFitting(u32),
}

//takes in a LayoutQuery and the global bottom of the viewport. 
//Returns the FitResult, relative to the given LayoutQuery
pub fn last_fitting_char(layout: &LayoutQuery, page_bottom: f64) -> FitResult {
    if layout.text_len() ==0 {
        return FitResult::AllFit
    }

    for child_idx in 0..layout.children.len(){
        let current_child = &layout.children[child_idx];
        match last_fitting_char(current_child, page_bottom) {
            FitResult::AllFit => continue,
            FitResult::NoneFit => return extract_previous_child_len(layout, child_idx),
            FitResult::LastFitting(i) => return FitResult::LastFitting(current_child.char_start + i-layout.char_start),
        }
    }

    if !layout.children.is_empty(){
        return FitResult::AllFit
    }

    let mut best_bottom=FitResult::NoneFit;
    for i in 0..layout.text_len() {
        let char_bottom = (layout.get_char_bottom)(i);
        console(&format!("checking: {}, got bottom: {}", i, char_bottom));
        if char_bottom <= page_bottom {
            best_bottom = FitResult::LastFitting(i);
        } else {
            break;
        }
    }

    if best_bottom == FitResult::LastFitting(layout.text_len()-1){
        return FitResult::AllFit;
    }

   best_bottom
}


fn extract_previous_child_len(layout: &LayoutQuery, idx: usize)->FitResult{
    if idx==0{
        return FitResult::NoneFit
    }
    if idx >=layout.children.len(){
        return FitResult::AllFit
    }

    let prev_child=&layout.children[idx-1];
    FitResult::LastFitting(prev_child.char_start+prev_child.text_len()-layout.char_start-1)
}


pub fn first_fitting_char(layout: &LayoutQuery, page_top: f64) -> FitResult {
    if layout.text_len() ==0 {
        return FitResult::AllFit
    }

    let mut best_top=FitResult::NoneFit;
    for i in (0..layout.text_len()).rev() {
        let char_top = (layout.get_char_top)(i);
        console(&format!("checking: {}, got top: {}", i, char_top));
        if char_top >= page_top {
            best_top = FitResult::LastFitting(i);
        } else {
            break;
        }
    }

    if best_top == FitResult::LastFitting(0){
        return FitResult::AllFit;
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