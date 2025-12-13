use dioxus::{hooks::{use_context, use_effect, use_signal},  signals::{Signal, WritableExt}};
use wasm_bindgen_futures::spawn_local;

use crate::models::{BookStatus, GlobalState};
use crate::components::server_api;

pub fn global_updater(){
    let global=use_context::<Signal<GlobalState>>();
    let mut old: Signal<Option<BookStatus>>=use_signal(||None);
    let mut updating=use_signal(||false);
    use_effect(move ||{
        
        let Some(book)=global().book else {return;};
        let Some(access_token)=global().access_token.clone() else {return;};
        match old() {
            None=>{
                updating.set(true);
                send_update(book.clone(), updating,access_token);
                old.set(Some(book));
            },
            Some(ob)=>{
                if ob != book && !updating(){
                    updating.set(true);
                    send_update(book.clone(),updating, access_token);
                    old.set(Some(book));
                }
            }
        }
    });
}

fn send_update(mut book:BookStatus, mut updating: Signal<bool>, access_token:String){
    spawn_local(async move{
        book.chapter=book.chapter.clamp(book.initial_chapter,book.max_chapter);
        if let Some(max) = book.chapter_to_chunk.get(&book.chapter) {
            book.chunk = book.chunk.clamp(1, *max);
        }
        let _ =server_api::update_progress(book,access_token).await;
        updating.set(false);
    });
}

