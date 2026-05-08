use dioxus::{hooks::{use_effect, use_signal}, logger::tracing, prelude::*, signals::{Signal, WritableExt}};
use html_escape::decode_html_entities;
use web_sys::{DomRect, HtmlElement};
use wasm_bindgen::{JsCast, prelude::Closure};
use crate::{domain::{self, text::{TextHandler, TextMap, find_sentence_offset_with_html_backtrack}}, infra};



/// Intent for alignment
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Align {
    Top,
    None,
    Bottom,
}

/// Render result returned by renderer
pub struct RenderResult {
    pub start_offset: usize,
    pub end_offset: usize,
    pub at_chapter_start: bool,
    pub at_chapter_end: bool,
}

pub async fn fetch_ch(book_id:String, idx: usize, mut chapter: Signal<String>, mut map:Signal<TextMap>){
    let html = infra::chapters::fetch_chapter(&book_id, idx).await;
    match html {
        Ok(txt) => {
            chapter.set(decode_html_entities(&domain::text::replace_html_entities(&txt)).to_string());
            
            map.set(domain::text::build_text_map_from_html(&chapter()));
        }
        Err(e) => tracing::error!("error in getting chapter: {}", e),
    }
}

pub fn use_renderer(book_id:String, mut align: Signal<Align>)->TextHandler{
    let text_handler=TextHandler::new(book_id.clone());
    let text=text_handler.clone();
    use_effect(move ||{
        let mut text=text.clone();
        let book_id=book_id.clone();
        spawn(async move {
           let cursor=infra::chapters::fetch_cursor_text(&book_id).await;
           match cursor {
               Err(_)=>align.set(Align::None),
               Ok(txt)=>{
                    text.chapter_idx.set(txt.cursor.cursor.chapter);
                    fetch_ch(text.book_id.clone(), (text.chapter_idx)(), text.chapter.clone(), text.map.clone()).await;
                    let offset=find_sentence_offset_with_html_backtrack(&txt.text, &(text_handler.map)());
                    text.start_offset.set(offset);
                    align.set(Align::Top);
               }
           }
        });
    });

    return text_handler;
}

pub fn render(text: &TextHandler, align: Signal<Align>, book_id:String){
    let mut text: TextHandler=text.clone();
    let book_s=use_signal(||book_id);
    
    use_effect(move||{
        if align() == Align::None {return;}
        if (text.chapter)().len() > 0{
            tracing::debug!("start: {}", (text.start_offset)());
            let book_id: String=book_s();
            spawn(async move{
                if (text.chapter_end)() && align()==Align::Top{
                    text.chapter_end.set(false);
                    text.chapter_idx.set((text.chapter_idx)()+1);
                    fetch_ch(book_id.clone(), (text.chapter_idx)(), text.chapter.clone(), text.map.clone()).await;
                    text.start_offset.set(0);
                    text.chapter_start.set(true);
                }else if (text.chapter_start)() && align()== Align::Bottom{
                    if (text.chapter_idx)() ==0 {return;}
                    text.chapter_start.set(false);
                    text.chapter_idx.set((text.chapter_idx)()-1);
                    fetch_ch(book_id.clone(), (text.chapter_idx)(), text.chapter.clone(), text.map.clone()).await;
                    text.start_offset.set((text.chapter)().len()-1);
                    text.chapter_end.set(true);
                }
                render_page(&(text.chapter)(), (text.start_offset)(), align(), (text.visible_text).clone());
            });
        }
    });
    let mut prev_start=use_signal(||0);
    
    use_effect(move||{
        let book_id=book_s.clone();
        let start=(text.start_offset)();
        if start!=prev_start(){
            save((text.chapter)(), book_id(), (text.chapter_idx)(), start);
            prev_start.set(start);
        }
    });

    use_effect(move || {
        if (text.visible_text)().is_empty() {
            return;
        }

        let mut text = text.clone();
        let mut align = align.clone();

        let closure = Closure::once_into_js(move || {
            let current_align = align();
            if current_align == Align::None {
                return;
            }

            // --- SNAPSHOT VALUES FIRST ---
            let chapter = (text.chapter)();
            let start_offset = (text.start_offset)();
            let visible_text = (text.visible_text).clone();


            tracing::debug!("run cut");

            let result = cut_render(
                &chapter,
                start_offset,
                current_align,
                visible_text,
            );

            tracing::debug!("results: {}", result.end_offset);

            // --- MUTATE SIGNALS AFTER ---
            text.start_offset.set(result.start_offset);
            text.end_offset.set(result.end_offset);
            text.chapter_end.set(result.at_chapter_end);
            text.chapter_start.set(result.at_chapter_start);
            align.set(Align::None);
        });

        if let Some(window) = web_sys::window() {
            window
                .request_animation_frame(
                    closure.as_ref().unchecked_ref()
                )
                .ok();
        }
    });
}

fn render_page(chapter_html: &str, start_offset: usize, align: Align,mut visible_text: Signal<String>){
    let slice = match align {
        Align::None => &chapter_html[start_offset..],
        Align::Top => &chapter_html[start_offset..],
        Align::Bottom => &chapter_html[..start_offset],
    };
    visible_text.set(slice.to_owned());
}


pub fn cut_render(
    chapter_html: &str,
    mut start_offset: usize,
    align: Align,
    mut visible_text: Signal<String>,
) -> RenderResult {
    let document = web_sys::window().unwrap().document().unwrap();
    let container = document
        .get_element_by_id("book-renderer")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    match align {
        Align::Top => {
            container.set_scroll_top(0);
            // Use our improved fallback mechanism
            let end_offset = find_proper_cut_point(&container, chapter_html, align, start_offset);
            
            tracing::debug!("end offset:{}", end_offset);

            // Ensure proper bounds - make sure we don't exceed text length
            let actual_end = end_offset.min(chapter_html.len());
            let actual_start = start_offset.min(actual_end);
            
            // Make sure we don't render empty content
            if actual_start >= actual_end {
                tracing::warn!("Invalid cut range - returning start position");
                return RenderResult {
                    start_offset: actual_start,
                    end_offset: actual_start,
                    at_chapter_start: actual_start == 0,
                    at_chapter_end: actual_start == chapter_html.len(),
                };
            }
            
            let html_fragment = &chapter_html[actual_start..actual_end];
            visible_text.set(html_fragment.to_owned());

            RenderResult {
                start_offset: actual_start,
                end_offset: actual_end,
                at_chapter_start: actual_start == 0,
                at_chapter_end: actual_end == chapter_html.len(),
            }
        }
        Align::None => unreachable!("Cut render should never be called without alignment"),
        Align::Bottom => {
            container.set_scroll_top(container.scroll_height());
            // Use our improved fallback mechanism
            let start_offset_result = find_proper_cut_point(&container, chapter_html, align, start_offset);
            
            tracing::debug!("back_result: {}-{}", start_offset_result, start_offset);

            // Ensure proper bounds
            let actual_start = start_offset_result.min(start_offset);
            let actual_end = start_offset;
            
            // Make sure we don't render empty content
            if actual_start >= actual_end {
                tracing::warn!("Invalid cut range - returning start position");
                return RenderResult {
                    start_offset: actual_start,
                    end_offset: actual_start,
                    at_chapter_start: actual_start == 0,
                    at_chapter_end: actual_start == chapter_html.len(),
                };
            }
            
            let html_fragment = &chapter_html[actual_start..actual_end];
            visible_text.set(html_fragment.to_owned());

            RenderResult {
                start_offset: actual_start,
                end_offset: actual_end,
                at_chapter_start: actual_start == 0,
                at_chapter_end: actual_end == chapter_html.len(),
            }
        }
    }
}

// Improved method to find cut points that handles HTML properly
fn find_proper_cut_point(container: &HtmlElement, chapter_html: &str, align: Align, start_offset: usize) -> usize {
    // First try the existing scanning logic
    if let Some(offset) = scan_children(container, chapter_html, align) {
        tracing::debug!("Found element-based cut point: {}", offset);
        return offset;
    }
    
    // Fallback to simple text-based cutting when element scanning fails
    tracing::warn!("Element scanning failed, using fallback text cutting");
    
    // For Top alignment, we want to find the end of viewport content
    if align == Align::Top {
        // Try to find a reasonable cut point based on viewport height
        let container_rect = container.get_bounding_client_rect();
        let container_height = container_rect.height();
        
        // Estimate how many characters can fit in the viewport
        // This is a rough estimate - in a real app, you'd measure actual text rendering
        let estimated_chars_per_line = 80; // Average chars per line
        let estimated_lines_per_viewport = (container_height / 20.0) as usize; // Rough estimation of 20px per line
        let estimated_chars_in_viewport = estimated_chars_per_line * estimated_lines_per_viewport;
        
        // Find a reasonable boundary in the text
        let mut end_offset = (start_offset + estimated_chars_in_viewport).min(chapter_html.len());
        
        // Try to find a sentence boundary or HTML tag boundary to avoid cutting in the middle of words or tags
        end_offset = find_sentence_boundary(chapter_html, start_offset, end_offset);
        
        tracing::debug!("Text fallback cut point: {}", end_offset);
        return end_offset;
    } else if align == Align::Bottom {
        // For bottom alignment, we want to start from the viewport bottom
        let container_rect = container.get_bounding_client_rect();
        let container_height = container_rect.height();
        
        // Estimate how many characters to go back
        let estimated_chars_per_line = 80;
        let estimated_lines_per_viewport = (container_height / 20.0) as usize;
        let estimated_chars_in_viewport = estimated_chars_per_line * estimated_lines_per_viewport;
        
        let mut start_offset_result = (start_offset.saturating_sub(estimated_chars_in_viewport)).min(chapter_html.len());
        
        // Try to find a sentence boundary or HTML tag boundary
        start_offset_result = find_sentence_boundary_backwards(chapter_html, start_offset_result, start_offset);
        
        tracing::debug!("Text fallback cut point (bottom): {}", start_offset_result);
        return start_offset_result;
    }
    
    // Fallback - return the start_offset if everything fails
    start_offset
}

// Find sentence boundary (look for sentence-ending punctuation)
fn find_sentence_boundary(text: &str, start: usize, max: usize) -> usize {
    let mut end = max.min(text.len());
    
    // Look for sentence endings like periods, exclamation marks, question marks
    let text_slice = &text[start..end];
    
    // Look for sentence endings backwards in the text slice
    for i in (0..text_slice.len()).rev() {
        let pos = start + i;
        if pos < text.len() {
            let ch = text.chars().nth(pos);
            if let Some(c) = ch {
                if c == '.' || c == '!' || c == '?' {
                    // Make sure we're not inside an HTML tag
                    let before_pos = pos.saturating_sub(20);
                    let before_text = &text[before_pos..pos];
                    if !before_text.contains('<') || !before_text.contains('>') {
                        return pos + 1; // Include the punctuation
                    }
                }
            }
        }
    }
    
    // If no sentence boundary found, look for a reasonable HTML tag boundary
    for i in (0..text_slice.len()).rev() {
        let pos = start + i;
        if pos < text.len() && text[pos..].starts_with("</") {
            return pos;
        }
    }
    
    // Fallback to simple cut at word boundary (but make sure we don't cut at the very start)
    let mut cut_pos = end;
    if cut_pos > start {
        while cut_pos > start && !text.chars().nth(cut_pos).map_or(false, |c| c.is_whitespace()) {
            cut_pos -= 1;
        }
        
        if cut_pos == start && end > start {
            // If we couldn't find whitespace, cut at a reasonable position
            cut_pos = (start + (end - start) / 2).min(end);
        }
    }
    
    if cut_pos == start {
        return end; // No space found, just cut at max
    }
    
    cut_pos
}

// Find sentence boundary backwards (for bottom alignment)
fn find_sentence_boundary_backwards(text: &str, start: usize, max: usize) -> usize {
    // Search forward from start to max looking for sentence endings
    let mut pos = start;
    
    while pos < max && pos < text.len() {
        if pos > 0 {
            let ch = text.chars().nth(pos);
            if let Some(c) = ch {
                if c == '.' || c == '!' || c == '?' {
                    // Make sure we're not inside an HTML tag
                    let after_pos = (pos + 20).min(text.len());
                    let after_text = &text[pos..after_pos];
                    if !after_text.contains('<') || !after_text.contains('>') {
                        return pos + 1; // Include the punctuation
                    }
                }
            }
        }
        pos += 1;
    }
    
    // If no sentence boundary found, look for HTML tag boundary
    pos = start;
    while pos < max && pos < text.len() {
        if text[pos..].starts_with('<') && text[pos..].starts_with("</") {
            return pos;
        }
        pos += 1;
    }
    
    // Fallback to reasonable cut point
    let cut_pos = (start + (max - start) / 2).min(max);
    if cut_pos > start {
        cut_pos
    } else {
        start
    }
}

fn save(chapter_html: String, book_id: String, index: usize, start: usize) {
    if start >= chapter_html.chars().count() {
        tracing::warn!("save skipped: start out of bounds ({})", start);
        return;
    }

    // Safely take a slice by chars
    let mut slice: String = chapter_html.chars().skip(start).take(1000).collect();

    let first_lt = slice.find('<');
    let first_gt = slice.find('>');

    if let Some(gt_pos) = first_gt {
        let should_trim = match first_lt {
            Some(lt_pos) => gt_pos < lt_pos,
            None => true,
        };

        if should_trim {
            slice = slice[gt_pos + 1..].to_string();
        }
    }


    if slice.len() > 50 {
        save_cursor(book_id, index, slice);
    }
}


fn scan_children(container: &HtmlElement, chapter_html: &str, align:Align)->Option<usize>{
    let container_rect = container.get_bounding_client_rect();
    return scan_children_inner(container, chapter_html, align, &container_rect);
    fn scan_children_inner(container: &HtmlElement, chapter_html: &str, align:Align, container_rect: &DomRect)->Option<usize>{
        let children = container.child_nodes(); 
        const EPSILON: f64 = 1.0;

        tracing::debug!("children length: {}", children.length());

        for i in 0..children.length(){
            let node = match children.item(i) {
                Some(n) => n,
                None => continue,
            };

            let el = match node.dyn_into::<HtmlElement>() {
                Ok(e) => e,
                Err(_) => continue,
            };

            if is_container(&el){
                tracing::debug!("div");
                if let Some(offset)=scan_children_inner(&el, chapter_html, align, container_rect){
                    return Some(offset);
                }else{
                    tracing::debug!("no val");
                    continue;
                }
            }


            let rect = el.get_bounding_client_rect();


            if rect.height() > 2.0 * container_rect.height() {
                // too big — skip
                continue;
            }
            let measurement=match align {
                Align::Bottom => first_fitting_child(&rect, &container_rect, EPSILON, i),
                Align::Top => first_overflowing_child(&rect, &container_rect, EPSILON, i),
                Align::None=>unreachable!("scan_children called with Align::None")
            };


            if let Some(idx) =measurement {
                if idx==children.length(){return None;}

                tracing::info!("index found: {}", idx);
                tracing::info!("search in order: {:?}", align);
                let el = children
                    .item(idx)
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap();
                let decoded_el=decode_html_entities(&el.outer_html()).to_string();
                tracing::info!("el_decoded: {}", decoded_el);
                tracing::info!("chapter_decooded: {}", chapter_html);
                // Try to find the actual position in the HTML
                if let Some(pos) = chapter_html.find(&decoded_el) {
                    return Some(pos);
                } else {
                    // If exact match not found, try to find where this element would be in the text
                    // This is a fallback - when an element is found but no position is found in text, 
                    // we return None to let fallback logic take over
                    tracing::warn!("Found element but couldn't locate in text - falling back to text cutting");
                    return None;
                }
            }else{
                tracing::info!("no measurement")
            }

        }
        None
    }
}



fn is_container(el: &HtmlElement) -> bool {
    // Treat block-level elements with multiple children as containers
    let block_tags = ["DIV", "SECTION", "ARTICLE", "BLOCKQUOTE"];
    block_tags.contains(&el.tag_name().as_str()) && el.child_nodes().length() > 0
}

fn first_overflowing_child(rect: &DomRect, container_rect: &DomRect, epsilon: f64, index:u32) -> Option<u32> {
    let bottom = rect.bottom();
    let container_bottom = container_rect.bottom();


    if bottom <= container_bottom - epsilon {
        return None;
    }

    tracing::info!("found overflow");
    Some(index)
}

fn first_fitting_child(rect: &DomRect, container_rect: &DomRect, epsilon: f64, index: u32) -> Option<u32> {
    let top = rect.top();
    let bottom = rect.bottom();
    let container_top = container_rect.top();
    let container_bottom = container_rect.bottom();

    if top >= container_top - epsilon && bottom <= container_bottom + epsilon {
        return Some(index);
    }

    None
}

pub fn save_cursor(book_id:String,index: usize, save_txt:String){
    spawn(async move{
        let cursor=infra::cursor::get_cursor_from_text(&book_id, index, &save_txt).await;
        match cursor {
            Err(e)=>tracing::error!("No cursor found: {}",e),
            Ok(c)=>{domain::cursor::save_bookcursor(c).await;}
        }
    });
}
