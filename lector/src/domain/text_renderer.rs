use dioxus::{hooks::{use_effect, use_signal}, logger::tracing, prelude::*, signals::{Signal, WritableExt}};
use once_cell::sync::Lazy;
use regex::Regex;
static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());


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
            chapter.set(txt.clone());
            
            map.set(domain::text::build_text_map_from_html(&txt));
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

pub fn render(text: &TextHandler, mut align: Signal<Align>, book_id:String){
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
            let map = (text.map)();
            let visible_text = (text.visible_text).clone();
            let chapter_idx = (text.chapter_idx)();
            let book_id = text.book_id.clone();

            tracing::debug!("run cut");

            let result = cut_render(
                &chapter,
                start_offset,
                current_align,
                visible_text,
                &map,
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
    start_offset: usize,
    align: Align,
    mut visible_text: Signal<String>,
    text_map: &domain::text::TextMap
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
            let end_offset = if let Some(child) = scan_children(&container, chapter_html, align) {
                child
            } else {
                tracing::debug!("found end");
                chapter_html.len()-1
            };

            tracing::debug!("end offset:{}",end_offset);

            let html_fragment = &chapter_html[start_offset..end_offset];
            visible_text.set(html_fragment.to_owned());

            RenderResult {
                start_offset,
                end_offset,
                at_chapter_start: start_offset == 0,
                at_chapter_end: end_offset == chapter_html.len()-1,
            }
        }
        Align::None => unreachable!("Cut renrer should never be called without alignment"),
        Align::Bottom => {
            container.set_scroll_top(container.scroll_height());
            let start_offset_result = if let Some(child) = scan_children(&container, chapter_html, align) {
                child
            } else {
                0
            };
            tracing::debug!("back_result: {}-{}", start_offset_result,start_offset);
            let html_fragment = &chapter_html[start_offset_result..start_offset];
            visible_text.set(html_fragment.to_owned());

            RenderResult {
                start_offset: start_offset_result,
                end_offset: start_offset,
                at_chapter_start: start_offset_result == 0,
                at_chapter_end: start_offset == chapter_html.len()-1,
            }
        }
    }
}


fn save(chapter_html: String, book_id: String, index: usize, start: usize) {
    if start >= chapter_html.len() {
        tracing::warn!("save skipped: start out of bounds ({})", start);
        return;
    }

    let mut text = normalize_text(slice_safe_html(&chapter_html, start, 1000));

    text = text.chars().take(200).collect();

    if text.len() > 50{
        save_cursor(book_id, index, text);
    }
}



fn scan_children(container: &HtmlElement, chapter_html: &str, align:Align)->Option<usize>{

    let container_rect = container.get_bounding_client_rect();
    return scan_children_inner(container, chapter_html, align, &container_rect);
    fn scan_children_inner(container: &HtmlElement, chapter_html: &str, align:Align, container_rect: &DomRect)->Option<usize>{
        let children = container.child_nodes(); 
        const EPSILON: f64 = 1.0;

        tracing::debug!("children lengt: {}", children.length());

        let iter: Box<dyn Iterator<Item = u32>> = if align==Align::Bottom {
            Box::new((0..children.length()).rev())
        } else {
            Box::new(0..children.length())
        };

        for i in iter{
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
                Align::Bottom => last_fitting_child(&rect, &container_rect, EPSILON, i),
                Align::Top => first_overflowing_child(&rect, &container_rect, EPSILON, i),
                Align::None=>unreachable!("scan_children called with Align::None")
            };


            if let Some(idx) =measurement {
                if idx==children.length(){return None;}

                tracing::debug!("index found: {}", idx);
                tracing::debug!("search in order: {:?}", align);
                let el = children
                    .item(idx)
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap();

                tracing::debug!("el: {}", el.outer_html());
                return chapter_html.find(&el.outer_html());
            }else{
                tracing::debug!("no measurement")
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

fn first_overflowing_child(rect: &DomRect, container_rect: &DomRect, epsilon: f64, index:u32)->Option<u32>{
    tracing::debug!("child: {}-{}",rect.top(),rect.bottom());
    tracing::debug!("container: {}-{}",container_rect.top(), container_rect.bottom());
    if rect.bottom() <= container_rect.bottom() + epsilon {
        return None;
    } else if rect.top() >= container_rect.bottom() - epsilon {
        return Some(index);
    } else {            
        return  Some(index);
    }

}

fn last_fitting_child(rect: &DomRect, container_rect: &DomRect, epsilon: f64, index:u32) -> Option<u32> {
    tracing::debug!("child: {}-{}",rect.top(),rect.bottom());
    tracing::debug!("container: {}-{}",container_rect.top(), container_rect.bottom());

    if rect.bottom() <= container_rect.bottom() + epsilon && rect.top() >= container_rect.top() - epsilon{
        return None;
    } else if rect.top() < container_rect.top() - epsilon {
        return Some(index+1);
    }else{
        return Some(index+1);
    }
}

fn check_char_match(c:&char)->bool{
    matches!(c, '.'| ' ' | '!' | '?' | '…' | '"' | '\'' | '“' | '”' | ',')
}

fn normalize_text(s: &str) -> String {
    HTML_TAG_RE.replace_all(s, "").to_string().chars()
        .filter(|c| !check_char_match(c))
        .collect::<String>()
        .trim()
        .to_lowercase()
}

pub fn save_cursor(book_id:String,index: usize, save_txt:String){
    spawn(async move{
        let cursor=infra::cursor::get_cursor_from_text(&book_id, index, &save_txt).await;
        match cursor {
            Err(e)=>tracing::error!("No cursor founnd: {}",e),
            Ok(c)=>{domain::cursor::save_bookcursor(c).await;}
        }
    });
}

fn slice_safe_html(s: &str, start: usize, max_len: usize) -> &str {
    let mut inside_tag = false;
    let mut char_count = 0;
    let mut start_byte = None;
    let mut end_byte = None;

    for (i, c) in s.char_indices() {
        // Track whether we are inside a tag
        if c == '<' { inside_tag = true; }
        if !inside_tag {
            // Only count chars outside tags
            if char_count >= start && start_byte.is_none() {
                start_byte = Some(i);
            }
            if char_count >= start + max_len {
                end_byte = Some(i);
                break;
            }
            char_count += 1;
        }
        if c == '>' { inside_tag = false; }
    }

    let start_byte = start_byte.unwrap_or(s.len());
    let end_byte = end_byte.unwrap_or(s.len());

    &s[start_byte..end_byte]
}