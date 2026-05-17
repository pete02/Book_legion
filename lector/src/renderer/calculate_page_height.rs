use std::sync::Arc;
use crate::renderer::sentence_boundaries;

#[derive(Clone)]
pub struct LayoutQuery{
    pub text: String,
    pub top: f64,
    pub bottom: f64,
    pub char_start: u32,
    pub children: Vec<LayoutQuery>,
    pub get_char_bottom: Arc<dyn Fn(u32) -> f64 + Send + Sync>,
    pub get_char_top: Arc<dyn Fn(u32) -> f64 + Send + Sync>,

}
impl LayoutQuery {
    pub fn text_len(&self) -> u32 {
        self.text.chars().count() as u32
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
            .field("top", &self.top)
            .field("bottom", &self.bottom)
            .field("char_start", &self.char_start)
            .field("children", &self.children)
            .finish()
    }
}



// Returns AllFit, if the layout fits and there is space left. If the Layout does not fit at all, returns NoneFit
//  If it partially fits, returns the index of the last fitting char.

pub fn last_fitting_char(layout: &LayoutQuery, page_height: f64) -> FitResult {
    if layout.top > page_height {return FitResult::NoneFit;}
    if layout.bottom == page_height {return FitResult::LastFitting(layout.char_start + layout.text_len() - 1);}
    if layout.bottom < page_height {return FitResult::AllFit;}
    if layout.children.is_empty() && layout.text_len() == 0 {return FitResult::AllFit;} 
    if page_height <= 0.0 {return FitResult::NoneFit;}

    for child in &layout.children {
        let child_fit = last_fitting_char(child, page_height);
        if child_fit != FitResult::AllFit && child_fit != FitResult::NoneFit {
            return child_fit;
        }
    }

    let mut best_bottom=FitResult::NoneFit;
    for i in 0..layout.text_len() {
        let char_bottom = (layout.get_char_bottom)(i);
        if char_bottom <= page_height {
            best_bottom = FitResult::LastFitting(layout.char_start+i);
        } else {
            break;
        }
    }

   best_bottom
}


pub fn first_fitting_char(layout: &LayoutQuery, page_top: f64) -> FitResult {
    if layout.bottom < page_top { return FitResult::NoneFit; }
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

pub fn last_fitting_sentence_boundary_cut(layout: &LayoutQuery, page_height: f64) -> Option<usize> {
    match last_fitting_char(layout, page_height) {
        FitResult::AllFit => Some(layout.subtree_end_char() as usize),
        FitResult::NoneFit => None,
        FitResult::LastFitting(last_char_index) => {
            let (text, local_char_index) = layout
                .find_leaf_text_for_char_index(last_char_index)?;
            let limit_byte = text
                .char_indices()
                .nth(local_char_index as usize)
                .map(|(byte_index, ch)| byte_index + ch.len_utf8())
                .unwrap_or(text.len());
            sentence_boundaries::find_last_sentence_boundary(text, limit_byte)
                .map(|boundary_byte| text[..boundary_byte].chars().count() as usize + (last_char_index - local_char_index) as usize)
        }
    }
}

pub fn first_fitting_sentence_boundary_cut(layout: &LayoutQuery, page_top: f64) -> Option<usize> {
    match first_fitting_char(layout, page_top) {
        FitResult::AllFit => Some(layout.char_start as usize),
        FitResult::NoneFit => None,
        FitResult::LastFitting(first_char_index) => {
            let (text, local_char_index) = layout
                .find_leaf_text_for_char_index(first_char_index)?;
            let start_byte = text
                .char_indices()
                .nth(local_char_index as usize)
                .map(|(byte_index, _)| byte_index)
                .unwrap_or(text.len());
            let limit_from_end = text.len().saturating_sub(start_byte);
            sentence_boundaries::find_first_sentence_boundary(text, limit_from_end)
                .map(|boundary_byte| text[..boundary_byte].chars().count() as usize + (first_char_index - local_char_index) as usize)
        }
    }
}
