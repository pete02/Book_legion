use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, window};

use crate::models::{BookStatus, GlobalState};


pub fn page_navigator(move_page:Signal<i32>, html_vec: Signal<Vec<String>>, visible_chunks: Signal<Vec<String>>){
    let private_fill=use_signal(||1);
    let visible_start=use_signal(||0);
 
    initial_chunk_sync(visible_start);
    chunk_filler(html_vec, visible_chunks, visible_start, private_fill);
    visible_start_updater(html_vec, visible_chunks, visible_start, private_fill);
    update_global_chunk(visible_start,visible_chunks,private_fill);
    page_turner(move_page, visible_start,visible_chunks, private_fill);
}

pub fn initial_chunk_sync(mut visible_start: Signal<i32>) {
    let mut started=use_signal(||false);
    use_effect(move || {
        if started() {return;};
        started.set(true);
        let global = use_context::<Signal<GlobalState>>();
        let Some(book) = global().book else { return };

        if visible_start() == 0 && book.chunk > 0 {
            tracing::debug!("book.chunk: {}", book.chunk);
            visible_start.set(book.chunk as i32 -1);
        }
    });
}


pub fn chunk_filler(
    html_vec: Signal<Vec<String>>,
    mut visible_chunks: Signal<Vec<String>>,
    visible_start:Signal<i32>,
    direction: Signal<i32>,
){
    let mut index: Signal<i32>=use_signal(||0);
    use_effect(move || {
        if visible_chunks().len() == 0{
            index.set(0);
        }
        let cur_location=visible_start()+index();

        if html_vec().len() ==0 {return;}
        if direction()==0 {return;}
        if is_full_height() {return;}
        if no_more_chunks(cur_location, direction(), html_vec) {return;}

        tracing::debug!("cur index: {}", cur_location);

        let chunk=html_vec()[cur_location as usize].clone();
        let mut v=visible_chunks().clone();

        if direction() ==1{
            v.push(chunk);
        }else{
            v.insert(0,chunk);
        }
        visible_chunks.set(v);
        index.set(index() + direction());
    });
}


fn visible_start_updater(
    html_vec: Signal<Vec<String>>,
    visible_chunks: Signal<Vec<String>>,
    mut visible_start:Signal<i32>,
    mut direction: Signal<i32>,
){
    use_effect(move || {
        let cur_location=visible_start()+visible_chunks().len() as i32;
        if html_vec().len() ==0 {return;}
        if direction()==0 {return;}
        if !(is_full_height() || no_more_chunks(cur_location, direction(), html_vec)){return;}

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
    let mut global=use_context::<Signal<GlobalState>>();
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
    mut move_page:Signal<i32>,
    mut visible_start:Signal<i32>,
    mut visible_chunks: Signal<Vec<String>>,
    mut private_move: Signal<i32>,
){
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move ||{
        let num=move_page();
        if num == 0 {return;};
        if private_move() != 0{return;};
        let Some(book)=global().book else {return;};
        if check_end(&book, move_page(), visible_start(), visible_chunks().len()) {return;}

        if move_page() ==1{
            let end_of_page = visible_start() + visible_chunks().len() as i32;
            let max_chunk=book.chapter_to_chunk[&book.chapter];
            if end_of_page >= max_chunk as i32{
                move_chapter(move_page());
                visible_start.set(0);
            }else{
                visible_start.set(visible_start()+visible_chunks().len() as i32);
            }
        }else{
            if visible_start()==0{
                visible_start.set(move_chapter(move_page()));
            }else{
                visible_start.set(visible_start() -1 );
            }
        }

        private_move.set(move_page());
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

    global.with_mut(|s|{
        let Some(book)=&mut s.book else {return;};
        book.chunk=1;
        book.chapter -=direction as u32;

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