use std::sync::Arc;

#[derive(Clone)]
pub struct LayoutQuery{
    pub text_len: u32,
    pub top: f64,
    pub bottom: f64,
    pub char_start: u32,
    pub children: Vec<LayoutQuery>,
    pub get_char_bottom: Arc<dyn Fn(u32) -> f64 + Send + Sync>,
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
            .field("text_len", &self.text_len)
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
    if layout.bottom == page_height {return FitResult::LastFitting(layout.char_start + layout.text_len - 1);}
    if layout.bottom < page_height {return FitResult::AllFit;}
    if layout.children.is_empty() && layout.text_len == 0 {return FitResult::AllFit;} 
    if page_height <= 0.0 {return FitResult::NoneFit;}

    for child in &layout.children {
        let child_fit = last_fitting_char(child, page_height);
        if child_fit != FitResult::AllFit && child_fit != FitResult::NoneFit {
            return child_fit;
        }
    }

    let mut best_bottom=FitResult::NoneFit;
    for i in 0..layout.text_len {
        let char_bottom = (layout.get_char_bottom)(i);
        if char_bottom <= page_height {
            best_bottom = FitResult::LastFitting(layout.char_start+i);
        } else {
            break;
        }
    }

   best_bottom
}