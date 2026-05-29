pub mod tests;
pub mod sentence_boundaries;
pub mod find_page_boundary;
pub mod layout_builder;
pub mod html_healer;
use web_sys::HtmlElement;
use crate::{infra, renderer::{find_page_boundary::{FitResult::{AllFit, LastFitting, NoneFit}, first_fitting_char_vec, last_fitting_char_vec}, html_healer::{decode_html, heal_html}, layout_builder::build_layout_vec}};


pub struct BookDriver {
    pub book_id: String,
    pub chapter_idx: usize,
    pub char_position: usize,
    viewport: HtmlElement,
    chapter_html: String,
    next_chapter_pending: bool,
}

impl BookDriver {
    pub async fn new(book_id: String, chapter_idx: usize, viewport: HtmlElement) -> Option<Self> {
        let chapter_html = match infra::chapters::fetch_chapter(&book_id, chapter_idx).await{
            Ok(text)=>text,
            Err(e)=>format!("<p>Error in getting chapter: {}</p>",e)
        };
        Some(Self {
            book_id,
            chapter_idx,
            char_position: 0,
            next_chapter_pending: false,
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
    async fn load_next_chapter(&mut self){
        self.chapter_idx += 1;
        self.char_position = 0;
        self.next_chapter_pending=false;

        match infra::chapters::fetch_chapter(&self.book_id, self.chapter_idx).await{
            Ok(chapter_text)=>self.chapter_html=decode_html(&chapter_text),
            Err(e)=>{
                self.chapter_html=format!("<p>Error in loading the chapter: {}</p>", e);
                self.chapter_idx -= 1;
                self.next_chapter_pending=true;

            }
        }
    }

    async fn drive_forward(&mut self){
        if self.next_chapter_pending {
            self.load_next_chapter().await;
        }
        let cut=cut_forward(&self.viewport, &self.chapter_html, self.char_position);
        match cut{
            None=>self.next_chapter_pending=true,
            Some(i)=>self.char_position=i
        }
    }
}

pub enum Direction {
    Forward,
    Back,
}



pub fn cut_forward(viewport: &HtmlElement, html: &str, char_start: usize)->Option<usize>{
    console("start forward");
    if html.len() ==0{
        return None;
    }

    let healed=html_healer::heal_html(&html[char_start..]);
    console(&format!("html: {}", healed));
    let rect=viewport.get_bounding_client_rect();
    viewport.set_inner_html(&healed);

    viewport.set_scroll_top(0);

    let layouts=build_layout_vec(viewport, &healed);
    let cutoff_res=last_fitting_char_vec(&layouts, rect.bottom());
    let cutoff=match cutoff_res{
        (AllFit,i)=>(layouts[i].char_start+layouts[i].text_len()) as usize,
        (NoneFit,_)=>0,
        (LastFitting(j),i)=>(layouts[i].char_start+j) as usize,
    };


    console(&format!("Cutoff: {}", cutoff));
    console(&healed[..cutoff]);

    let last=layouts.last().unwrap();
    if cutoff as u32== last.char_start+last.text_len(){
        return None
    }


    match sentence_boundaries::find_last_sentence_boundary(&healed, cutoff, 0) {
        Some(end) => {
            let complete_html=heal_html(&healed[..end]);
            viewport.set_inner_html(&complete_html);
            Some(char_start+end)
        },
        None => Some(char_start+cutoff),
    }
}


pub fn cut_backward(viewport: &HtmlElement, html: &str, char_end: usize)->Option<usize>{
    console(&format!("start backward: {}", char_end));
    if html.len() ==0{
        return None;
    }

    let healed=html_healer::heal_html(&html[..char_end]);
    console(&format!("html: {}", healed));
    let rect=viewport.get_bounding_client_rect();
    viewport.set_inner_html(&healed);

    viewport.set_scroll_top(viewport.scroll_height());
    console(&format!("scroll height: {}, viweport:{}", viewport.scroll_height(), rect.height()));

    let layouts=build_layout_vec(viewport, &healed);
    console(&format!("layout top: {}", (layouts[0].get_char_top)(0)));

    let cutoff_res=first_fitting_char_vec(&layouts, rect.top());
    let cutoff=match cutoff_res{
        (AllFit,i)=>(layouts[i].char_start) as usize,
        (NoneFit,_)=>char_end,
        (LastFitting(j),i)=>(layouts[i].char_start+j) as usize,
    };


    console(&format!("Cutoff: {}", cutoff));
    console(&healed[cutoff..]);

    let first=layouts.first().unwrap();
    if cutoff as u32== first.char_start{
        return None
    }


    match sentence_boundaries:: find_first_sentence_boundary(&healed, cutoff, healed.len()) {
        Some(start) => {
            console(&format!("start: {}", start));
            let complete_html=heal_html(&healed[start..]);
            console(&format!("complete: {}",complete_html ));
            viewport.set_inner_html(&complete_html);
            Some(start)
        },
        None => Some(cutoff),
    }
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
