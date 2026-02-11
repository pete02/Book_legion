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
            chapter.set(domain::text::replace_html_entities(&txt));
            
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
            let mut end_offset = if let Some(child) = scan_children(&container, chapter_html, align) {
                child
            } else {
                tracing::info!("found end");
                chapter_html.len()-1
            };

            tracing::debug!("end offset:{}",end_offset);

            if start_offset>end_offset{
                let temp=end_offset;
                end_offset=start_offset;
                start_offset=temp;
            }

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
            let mut start_offset_result = if let Some(child) = scan_children(&container, chapter_html, align) {
                child
            } else {
                0
            };
            tracing::debug!("back_result: {}-{}", start_offset_result,start_offset);

            if start_offset_result>start_offset{
                let temp=start_offset_result;
                start_offset_result=start_offset;
                start_offset=temp;
            }
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

        tracing::debug!("children lengt: {}", children.length());

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
                let decoded=decode_html_entities(&chapter_html).to_string();
                tracing::info!("el_decoded: {}", decoded_el);
                tracing::info!("chapter_decooded: {}", decoded);
                return decoded.find(&decoded_el);
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
            Err(e)=>tracing::error!("No cursor founnd: {}",e),
            Ok(c)=>{domain::cursor::save_bookcursor(c).await;}
        }
    });
}

