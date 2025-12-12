use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, window};

use crate::models::GlobalState;


pub fn page_navigator(move_page:Signal<i32>, html_vec: Signal<Vec<String>>, visible_chunks: Signal<Vec<String>>){
    let private_fill=use_signal(||1);
    let visible_start=use_signal(||0);
    let far_end=use_signal(|| false);
    

    initial_chunk_sync(visible_start);
    chunk_filler(html_vec, visible_chunks, visible_start, private_fill, far_end);
    update_global_chunk(visible_start,visible_chunks,private_fill, far_end, );
    page_turner(move_page, visible_start,visible_chunks, private_fill, far_end);
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
    visible_chunks: Signal<Vec<String>>,
    visible_start:Signal<i32>,
    book_move: Signal<i32>,
    mut far_end: Signal<bool>
){
    let mut visible_chunks=visible_chunks.clone();
    let mut index=use_signal(||0);
    use_effect(move || {
        if html_vec().len() == 0 {return;};
        if book_move() ==0 {return;};

        let mut v=visible_chunks();
        if is_full_height() {return;};

        if visible_chunks().len()==0 {
            tracing::debug!("reset index");
            index.set(0);
        }

        let next_index=(visible_start()+index()).clamp(0, html_vec.len() as i32);
        tracing::debug!("htlm_vec: {}", html_vec().len());
        tracing::debug!("book move: {}", book_move());

        if next_index ==0 && book_move() == -1 {
            tracing::debug!("farend");
            far_end.set(true);
            return;
        }
        if next_index >= html_vec.len() as i32 && book_move() == 1 as i32 {
            tracing::debug!("farend");
            far_end.set(true);
            return;
        };
        tracing::debug!("next index: {}", next_index);
        
        let chunk=html_vec()[next_index as usize].clone();
        
        if book_move() == -1{
            v.insert(0, chunk);
        }else{
            v.push(chunk);
        }

        index.set(index()+ book_move());
        visible_chunks.set(v);
    });
}

pub fn update_global_chunk(
    mut visible_start:Signal<i32>,
    visible_chunks: Signal<Vec<String>>,
    mut move_book:Signal<i32>,
    far_end: Signal<bool>
){
    let mut global=use_context::<Signal<GlobalState>>();
    use_effect(move || {
        let _=visible_chunks();
        let cur_index=visible_start() as u32;
        let Some(book)=global().book else {return;};
        if book.chunk== cur_index {return;};
        if !(is_full_height() || far_end()) {return;};


        tracing::debug!("trigger global");
        if move_book() == -1 {
            let val=visible_start()-visible_chunks().len() as i32 + 1;
            visible_start.set(val.max(0));
        }

        global.with_mut(|s|{
            let Some(book)=&mut s.book else {return;};
            book.chunk=visible_start() as u32
        });

        move_book.set(0);
    });
}



pub fn page_turner(
    mut move_page:Signal<i32>,
    mut visible_start:Signal<i32>,
    mut visible_chunks: Signal<Vec<String>>,
    mut private_move: Signal<i32>,
    far_end: Signal<bool>
){
    let mut global=use_context::<Signal<GlobalState>>();
    use_effect(move ||{
        let num=move_page();
        let Some(book)=global().book else {return;};
        tracing::debug!("check move");
        if private_move() != 0{return;};
        tracing::debug!("private ok");
        if num == 0 {return;};
        tracing::debug!("public ok");
        if move_page()==1 && book.chapter >= book.max_chapter && far_end() {return;};
        tracing::debug!("farend 1");
        if move_page()==-1 && book.chapter <= book.initial_chapter && far_end() {return;};

        tracing::debug!("move");

        if move_page() ==1{
            if far_end(){
                global.with_mut(|s|{
                    let Some(book)=&mut s.book else {return;};
                    book.chunk=1;
                    book.chapter +=1;
                });
                visible_start.set(0);
            }else{
                visible_start.set(visible_start()+visible_chunks().len() as i32);
            }
        }else{
            if far_end(){
                global.with_mut(|s|{
                    let Some(book)=&mut s.book else {return;};
                    book.chunk=1;
                    book.chapter -=1;
                });
                visible_start.set(0);
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


