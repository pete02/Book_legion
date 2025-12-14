use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, window};

use crate::models::{BookStatus, GlobalState};


pub fn page_navigator(move_page:Signal<i32>, html_vec: Signal<Vec<String>>, visible_chunks: Signal<Vec<String>>){
    let private_fill=use_signal(||1);
    let visible_start=use_signal(||0);
    let index: Signal<i32>=use_signal(||0);
    let stopped=use_signal(||false);

    initial_chunk_sync(visible_start);
    chunk_filler(html_vec, visible_chunks, visible_start, private_fill, index, stopped);
    visible_start_updater(html_vec,index, visible_start, visible_chunks,private_fill, stopped);
    update_global_chunk(visible_start,visible_chunks,private_fill);
    page_turner(html_vec, move_page, visible_start,visible_chunks, private_fill, stopped);
}

pub fn initial_chunk_sync(mut visible_start: Signal<i32>) {
    let mut started=use_signal(||false);
    let global = use_context::<Signal<GlobalState>>().clone();
    use_effect(move || {
        let Some(book) = global().book else { return };
        if started() {return;};
        started.set(true);

        tracing::debug!("run start");
        if visible_start() == 0 && book.chunk > 0 {
            tracing::debug!("book.chunk: {}", book.chunk);
            let val=book.chunk as i32 -4;
            visible_start.set(val.max(0));
        }
    });
}


pub fn chunk_filler(
    html_vec: Signal<Vec<String>>,
    mut visible_chunks: Signal<Vec<String>>,
    visible_start:Signal<i32>,
    direction: Signal<i32>,
    mut index: Signal<i32>,
    mut stopped: Signal<bool>
){

    use_effect(move || {
        if visible_chunks().len() == 0{
            index.set(0);            
        }
        let cur_location=visible_start()+index();
        if html_vec().len() ==0 {return;}
        if direction()==0 {return;}
        if stopped() {return;};
        if no_more_chunks(cur_location, direction(), html_vec) {
            stopped.set(true);
            return;
        }



        let i= (cur_location as usize).clamp(0, html_vec().len());
        let chunk=html_vec()[i].clone();
        let mut v=visible_chunks().clone();

        if direction() ==1{
            v.push(chunk);
        }else{
            v.insert(0,chunk);
        }
        visible_chunks.set(v);


        let mut v=visible_chunks().clone();
        if is_full_height() {
            stopped.set(true);
            if direction()==1{
                v.pop();
                visible_chunks.set(v);
            }else{
                v.remove(0);
                visible_chunks.set(v);
            }
            return;
        }


        index.set(index() + direction());
    });
}


fn visible_start_updater(
    html_vec: Signal<Vec<String>>,
    index:Signal<i32>,
    mut visible_start:Signal<i32>,
    visible_chunks: Signal<Vec<String>>,
    mut direction: Signal<i32>,
    stopped: Signal<bool>
){
    use_effect(move || {
        if html_vec().len() ==0 {return;}
        if visible_chunks.len() == 0 {return;}
        if direction()==0 {return;}
        if !stopped() {return;}

        if direction()==-1{
            let val=visible_start()-visible_chunks().len() as i32 + 1; visible_start.set(val.max(0));
            visible_start.set(val);
        }
        direction.set(0);

    });
}

pub fn update_global_chunk(
    visible_start:Signal<i32>,
    visible_chunks: Signal<Vec<String>>,
    move_book:Signal<i32>,
){
    let mut global=use_context::<Signal<GlobalState>>().clone();
    use_effect(move || {
        let _=visible_chunks();
        let cur_index=visible_start() as u32;

        let Some(book)=global().book else {return;};
        if book.chunk== cur_index {return;};
        if move_book()!=0{return;}

        global.with_mut(|s|{
            let Some(book)=&mut s.book else {return;};
            book.chunk=visible_start() as u32
        });
    });
}



pub fn page_turner(
    mut html_vec: Signal<Vec<String>>,
    mut move_page:Signal<i32>,
    mut visible_start:Signal<i32>,
    mut visible_chunks: Signal<Vec<String>>,
    mut private_move: Signal<i32>,
    mut stopped: Signal<bool>
){
    let global=use_context::<Signal<GlobalState>>().clone();
    use_effect(move ||{
        let num=move_page();
        if num == 0 {return;};
        if private_move() != 0{return;};
        let Some(book)=global().book else {return;};
        if check_end(&book, move_page(), visible_start(), visible_chunks().len()) {return;}
        tracing::debug!("move");
        if move_page() ==1{
            let end_of_page = visible_start() + visible_chunks().len() as i32;
            let max_chunk=book.chapter_to_chunk[&book.chapter];
            if end_of_page >= max_chunk as i32{
                move_chapter(move_page());
                visible_start.set(0);
                html_vec.set(Vec::new());
                
            }else{
                tracing::debug!("Da");
                visible_start.set(visible_start()+visible_chunks().len() as i32);
            }
        }else{
            if visible_start()==0{
                visible_start.set(move_chapter(move_page()));
                html_vec.set(Vec::new());
            }else{
                visible_start.set(visible_start() -1 );
            }
        }

        private_move.set(move_page());
        stopped.set(false);
        move_page.set(0);
        visible_chunks.set(Vec::new());
    });
}




fn is_full_height()->bool{
    let Some(nav_height)=get_element_height("book-container") else {return true;};
    let Some(book_height)=get_element_height("book-renderer") else {return true;};
    
    book_height >= nav_height-100.0
}

fn get_element_height(id: &str) -> Option<f64> {
    let document = window()?.document()?;
    let element = document.get_element_by_id(id)?;
    let html_element: HtmlElement = element.dyn_into().ok()?;
    Some(html_element.offset_height() as f64)
}


fn check_end(book: &BookStatus, direction: i32, cur_location: i32, possible_addition: usize) -> bool {
    let first = 0;
    if !book.chapter_to_chunk.contains_key(&book.chapter){
        tracing::error!("not contains: {}",book.chapter);
        tracing::error!("{:?}",book.chapter_to_chunk);
    }

    let last = book.chapter_to_chunk[&book.chapter] - 1;

    (direction == 1 && book.chapter == book.max_chapter && cur_location+possible_addition as i32 >= last as i32)
        ||
    (direction == -1 && book.chapter == book.initial_chapter && cur_location <= first)
}


fn move_chapter(direction: i32)->i32{
    let mut global=use_context::<Signal<GlobalState>>();
    let mut cor_chunk=1;

    tracing::debug!(" move chapter, direction: {}", direction);

    global.with_mut(|s|{
        let Some(book)=&mut s.book else {return;};
        book.chunk=1;
        let chap=book.chapter as i32 + direction;
        book.chapter=(chap as u32).clamp(book.initial_chapter, book.max_chapter);
        tracing::debug!("new ");
        if direction==-1{
            cor_chunk=book.chapter_to_chunk[&book.chapter]-1;
        }
    });

    cor_chunk as i32
}

fn no_more_chunks( cur_location: i32, direction: i32, html_vec: Signal<Vec<String>>)->bool{
    (cur_location==-1 && direction==-1)
    || 
    (direction==1 && cur_location == html_vec.len() as i32)
}