pub mod tests;
pub mod sentence_boundaries;
pub mod find_page_boundary;
pub mod layout_builder;
pub mod html_healer;



use dioxus::logger::tracing::{self, warn};
use web_sys::HtmlElement;
use crate::{infra, renderer::{find_page_boundary::{FitResult::{AllFit, LastFitting, NoneFit}, first_fitting_char_vec, last_fitting_char_vec}, html_healer::{decode_html, heal_html, slice_text}, layout_builder::build_layout_vec}};
use crate::domain;
#[derive(Debug, Clone)]
pub struct BookDriver {
    pub book_id: String,
    pub chapter_idx: usize,
    pub prev_char_position: usize,
    pub char_position: usize,
    pub viewport: HtmlElement,
    pub chapter_html: String,
    pub next_chapter_pending: bool,
}

impl BookDriver {
    pub async fn new(book_id: String, viewport: HtmlElement) -> Option<Self> {
        let cursor=domain::cursor::fetch_cursor_text(&book_id).await;


        let chapter_html = match infra::chapters::fetch_chapter(&book_id, cursor.cursor.cursor.chapter).await{
            Ok(text)=>decode_html(&text.replace("calibre", "replaced")),
            Err(e)=>format!("<p>Error in getting chapter: {}</p>",e)
        };

        let txtmap=domain::text::build_text_map_from_html(&chapter_html);
        let offset=domain::text::find_sentence_offset_with_html_backtrack(&cursor.text, &txtmap);
        console(&format!("offset: {}",offset));
        Some(Self {
            book_id,
            chapter_idx: cursor.cursor.cursor.chapter,
            char_position: offset,
            prev_char_position: offset,
            next_chapter_pending: false,
            viewport,
            chapter_html,
        })
    }

    pub async fn go(&mut self, direction: Direction) {
        match direction {
            Direction::Forward => self.drive_forward().await,
            Direction::Back => self.drive_backward().await,
        };
    }
    async fn load_next_chapter(&mut self){
        self.chapter_idx += 1;
        self.char_position = 0;
        self.prev_char_position=0;
        self.next_chapter_pending=false;

        match infra::chapters::fetch_chapter(&self.book_id, self.chapter_idx).await{
            Ok(chapter_text)=>self.chapter_html=decode_html(&chapter_text).replace("calibre", "replaced"),
            Err(e)=>{
                self.chapter_html=format!("<p>Error in loading the chapter: {}</p>", e);
                self.chapter_idx -= 1;
                self.next_chapter_pending=true;

            }
        }
    }

    async fn load_prev_chapter(&mut self){
        self.chapter_idx -= 1;
        self.next_chapter_pending=true;

        match infra::chapters::fetch_chapter(&self.book_id, self.chapter_idx).await{
            Ok(chapter_text)=>{
                self.chapter_html=decode_html(&chapter_text).replace("calibre", "replaced");
                self.prev_char_position=self.chapter_html.len();
            },
            Err(e)=>{
                self.chapter_html=format!("<p>Error in loading the chapter: {}</p>", e);
                self.chapter_idx += 1;
                self.next_chapter_pending=false;

            }
        }
    }

    async fn drive_forward_for_recursive(&mut self){
        if self.next_chapter_pending {
            self.load_next_chapter().await;
        }
        self.save().await;

        let cut=cut_forward(&self.viewport, &self.chapter_html, self.char_position);
        match cut{
            None=>{
                self.next_chapter_pending=true;
                self.prev_char_position=self.chapter_html.len();
                
            },
            Some(i)=>{
                if i==self.chapter_html.len(){
                    self.next_chapter_pending=true;
                    self.prev_char_position=self.chapter_html.len();
                }

                self.prev_char_position=self.char_position;
                self.char_position=i

            }
        }

    }

    async fn drive_forward(&mut self){
        if self.next_chapter_pending {
            self.load_next_chapter().await;
        }
        self.save().await;

        let cut=cut_forward(&self.viewport, &self.chapter_html, self.char_position);
        match cut{
            None=>{
                self.next_chapter_pending=true;
                self.prev_char_position=self.chapter_html.len();
                self.drive_forward_for_recursive().await
            },
            Some(i)=>{
                if i==self.chapter_html.len(){
                    self.next_chapter_pending=true;
                    self.prev_char_position=self.chapter_html.len();
                }

                self.prev_char_position=self.char_position;
                self.char_position=i

            }
        }

    }

    pub async fn save(&mut self) {
        save(&self.chapter_html, &self.book_id,self.chapter_idx,self.char_position).await;
    }

    async fn drive_backward(&mut self){
        tracing::debug!("Driving backward: {}: {}",self.chapter_idx, self.prev_char_position);
        self.save().await;
        if self.chapter_idx==0 && self.prev_char_position==0{
            self.char_position=0;
            tracing::debug!("start of book");
            self.drive_forward().await;
            return ;
        }

        if self.prev_char_position==0{
            tracing::debug!("load prev chapter");
            if self.chapter_idx==0{
                return ;
            }
            self.load_prev_chapter().await;
        }else{
            self.next_chapter_pending=false;
        }

        match cut_backward(&self.viewport, &self.chapter_html, self.prev_char_position) {
            Some(i) => {
                self.char_position = self.prev_char_position;
                self.prev_char_position = i;
            },
            None =>{
                self.char_position=self.prev_char_position;
                self.prev_char_position = 0
            },
        }
    }
}

pub enum Direction {
    Forward,
    Back,
}

use web_sys;

pub fn cut_forward(viewport: &HtmlElement, html: &str, char_start: usize)->Option<usize>{
    tracing::debug!("start forward");
    if html.len() ==0{
        return Some(html.len());
    }

    let rect=viewport.get_bounding_client_rect();
    viewport.set_inner_html(&html_healer::heal_html(slice_text(html, Some(char_start), None)));

    viewport.set_scroll_top(0);


    let layouts=build_layout_vec(viewport, &html);
    if layouts.len()==0{
        return None;
    }

    let cutoff_res=last_fitting_char_vec(&layouts, rect.bottom());
    let cutoff=match cutoff_res{
        (AllFit,i)=>(layouts[i].char_start+layouts[i].text_len()) as usize,
        (NoneFit,_)=>0,
        (LastFitting(j),i)=>(layouts[i].char_start+j) as usize,
    };


    let last=layouts.last().unwrap();
    if cutoff as u32== last.char_start+last.text_len(){
        warn!("no layout found");
        return Some(html.len())
    }

    tracing::debug!("set {}", slice_text(html, Some(char_start), Some(cutoff)));  

    tracing::info!("moving to find last sentence boundary");
    match sentence_boundaries::find_last_sentence_boundary(&html, cutoff, 0) {
        Some(end) => {
            let complete_html=heal_html(slice_text(html, Some(char_start), Some(end)));
            viewport.set_inner_html(&complete_html);
            Some(end)
        },
        None => Some(char_start+cutoff),
    }
}


pub fn cut_backward(viewport: &HtmlElement, html: &str, char_end: usize)->Option<usize>{


    tracing::debug!("start backward: {}", char_end);
    if html.len() ==0{
        return None;
    }

    let healed=html_healer::heal_html(&html[..char_end]);
    
    let rect=viewport.get_bounding_client_rect();
    viewport.set_inner_html(&healed);

    viewport.set_scroll_top(viewport.scroll_height());

    let layouts=build_layout_vec(viewport, &healed);

    let cutoff_res=first_fitting_char_vec(&layouts, rect.top());
    let cutoff=match cutoff_res{
        (AllFit,i)=>(layouts[i].char_start) as usize,
        (NoneFit,_)=>char_end,
        (LastFitting(j),i)=>(layouts[i].char_start+j) as usize,
    };


    tracing::debug!("Cutoff: {}", cutoff);

    let first=layouts.first().unwrap();
    if cutoff as u32== first.char_start{
        return None
    }


    tracing::debug!("finding first sentence boundary");
    match sentence_boundaries:: find_first_sentence_boundary(&healed, cutoff, healed.len()) {
        Some(start) => {
            tracing::debug!("start: {}", start);
            let complete_html: String=heal_html(slice_text(html, Some(start), Some(char_end)));
            viewport.set_inner_html(&complete_html);
            
            Some(start)
        },
        None => Some(0),
    }
}


async fn save(chapter_html: &str, book_id: &str, index: usize, start: usize){
    console(&format!("trest: {}",book_id));
    let slice=get_save_slice(chapter_html,  start);
    if slice.len() > 50 {
        console("saving");
        match domain::cursor::save_cursor_text(&book_id, &slice, index).await {
            Ok(()) => console("saved?"),
            Err(e) => console(&format!("Error saving cursor text: {}", e)),
        }
    }else{
        console(&format!("save skipped: slice too short ({}); given html: {}", slice.len(), chapter_html));
    }
}

pub fn get_save_slice(chapter_html: &str, start: usize)->String {
    
    if start >= chapter_html.chars().count() {
        console(&format!("save skipped: start out of bounds ({})", start));
        return String::new();
    }
    return chapter_html.chars().skip(start).take(1000).collect();
}


#[cfg(target_arch = "wasm32")]
use web_sys::console;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
pub fn console(text: &str){
    #[cfg(target_arch = "wasm32")]
    console::log_1(&JsValue::from_str(text));
    #[cfg(not(target_arch = "wasm32"))]
    println!("{}", text);
}

