pub mod tests;
pub mod sentence_boundaries;
pub mod calculate_page_height;
pub mod layout_builder;
pub mod html_healer;
use dioxus::html::view;
use web_sys::HtmlElement;
use crate::{infra, renderer::calculate_page_height::split_html_at};


pub struct BookDriver {
    pub book_id: String,
    pub chapter_idx: usize,
    pub char_position: usize,
    pub char_end: usize,
    viewport: HtmlElement,
    chapter_html: String,
}

impl BookDriver {
    pub async fn new(book_id: String, chapter_idx: usize, viewport: HtmlElement) -> Option<Self> {
        let chapter_html = infra::chapters::fetch_chapter(&book_id, chapter_idx).await.ok()?;
        Some(Self {
            book_id,
            chapter_idx,
            char_position: 0,
            char_end: 0,
            viewport,
            chapter_html,
        })
    }

    pub async fn go(&mut self, direction: Direction) {
        match direction {
            Direction::Forward => self.drive_forward().await,
            Direction::Back => todo!(),
        };
    }


    async fn drive_forward(&mut self){
        if self.char_end == self.chapter_html.len(){
            match infra::chapters::fetch_chapter(&self.book_id, self.chapter_idx + 1).await {
                Ok(html) => {
                    self.chapter_idx += 1;
                    self.chapter_html = html;
                    self.char_position = 0;
                    self.char_end = 0;
                    self.load_and_advance(0);
                }
                Err(e) => {
                    console(&format!("Failed to fetch next chapter: {:?}", e));
                    // Stay on current page — do nothing
                }
            }
        }else{
            self.load_and_advance(self.char_end);
        }
    }


    fn load_and_advance(&mut self, start: usize) {
        match load_chapter(&self.viewport, &self.chapter_html, start) {
            Some(end) => {
                self.char_position = start;
                self.char_end = end;
            }
            None => console("not implemented"),
        }
    }
}

pub enum Direction {
    Forward,
    Back,
}



pub fn load_chapter(viewport: &HtmlElement, htlm: &str, char_start: usize)->Option<usize>{
    let html = html_healer::heal_html(split_html_at(htlm, char_start));  
    console(&format!("Start the text at {}",char_start));
    viewport.set_inner_html(&html);
    let rect=viewport.get_bounding_client_rect().height();
    console(&format!("rect height: {}", rect));
    let layout=layout_builder::build_layout(viewport, char_start as u32, &html);
    let res=calculate_page_height::last_fitting_sentence_boundary_cut(&html,&layout, rect, char_start);
    if let Some(absolute_cutoff)=res{
        console(&format!("last fitting char: {}, start char: {}", absolute_cutoff, char_start));
        let relative_cutoff=absolute_cutoff-char_start;
        let fit=&html[..relative_cutoff as usize];
        console(&format!("fit len: {}", fit.len()));
        let fixed=html_healer::heal_html(fit);
            console(&format!("fit: {}", fixed));

        viewport.set_inner_html(&fixed);
        return Some(absolute_cutoff)
    }
    console("cutter returned None");
    return None
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
