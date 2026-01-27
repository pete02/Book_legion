use std::{collections::HashMap, time::Duration};


use dioxus::{ logger::tracing, prelude::*};
use gloo_timers::future::sleep;
use js_sys::{Array, Uint8Array};
use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

use crate::{domain::{self, cursor::{ BookCursor, Cursor}}, infra};



#[derive(Clone, PartialEq, Copy)]
pub struct AudioData{
    pub book_id:Signal<String>,
    pub name: Signal<String>,
    pub audio_url: Signal<String>,
    pub current_cursor: Signal<BookCursor>,
    pub playing: Signal<bool>,
    pub progress: Signal<f64>,
    pub audio_urls:Signal<HashMap<BookCursor,String>>,
    pub chapter_change: Signal<Vec<Cursor>>,
    pub guard: Signal<bool>,
    pub debounce: Signal<i32>
}

const AHEAD:usize=10;

fn error_audio(book_id: String)->AudioData{
    AudioData { 
        book_id: use_signal(||book_id),
        name: use_signal(||"error in getting audio data".to_string()),
        audio_url: use_signal(||"".to_owned()),
        progress: use_signal(||0.0),
        current_cursor: use_signal(||BookCursor::new("u1", "b1", 0, 0)),
        playing: use_signal(||false),
        audio_urls: use_signal(||HashMap::new()),
        chapter_change: use_signal(|| Vec::new()),
        guard: use_signal(||false),
        debounce: use_signal(||0)
    }
}

pub async fn load_audio(book_id:String, mut audio: AudioData){
    let book=domain::book::load_book(book_id.clone()).await;
    let cursor=domain::cursor::load_bookcursor(book_id.clone()).await;
    
    audio.book_id.set(book_id.clone());
    audio.name.set(book.title);
    audio.progress.set(domain::book::get_book_progress(book_id.clone()).await);
    audio.playing.set(false);
    audio.current_cursor.set(cursor);

    spawn(async move{
        get_audio_url(audio.clone()).await;
    });
}

pub fn use_audio(book_id: String) -> AudioData {
    let book =error_audio(book_id.clone());
    let v=book.clone();
    use_effect(move ||{
        let book=book.clone();
        let book_id=book_id.clone();
        spawn(async move{
            load_audio(book_id,book).await;
        });
    });
    v
}

fn cursors_ahead(audio: &AudioData) -> usize {
    (audio.audio_urls)().keys().filter(|c| **c > (audio.current_cursor)()).count()
}

pub async fn get_audio_url(mut audio: AudioData) {
    let current_cursor = (audio.current_cursor)();

    let audio_map = (audio.audio_urls)();

    if let Some(url) = audio_map.get(&current_cursor) {
        if cursors_ahead(&audio) < AHEAD {
            load_audio_urls(audio.clone());
        }
        audio.audio_url.set(url.clone());
    } else {
        get_audio_urls(audio.clone()).await;
        if let Some(url) = (audio.audio_urls)().get(&current_cursor) {
            audio.audio_url.set(url.clone());
        }
    }
    spawn(async move{
        domain::cursor::save_bookcursor(current_cursor.clone()).await;
        audio.progress.set(domain::book::get_book_progress((audio.book_id)()).await);
    });
}

fn load_audio_urls(mut audio:AudioData){
    if !(audio.guard)(){
        audio.guard.set(true);
        spawn(async move{get_audio_urls(audio).await});
    }
}


pub async fn get_audio_urls(mut audio:AudioData){
    match infra::audio::get_chunks((audio.current_cursor)(), AHEAD).await {
        Err(e)=>{
            tracing::error!("Error in loading audioo_urls: {}",e);
        },
        Ok(urls)=>{
            let mut c=(audio.current_cursor)();
            for e in urls{
                if e.cursor.cursor.chapter > c.cursor.chapter{
                    audio.chapter_change.with_mut(|f|f.push(c.cursor.clone()));
                }
                c.cursor=e.cursor.cursor;
                let url=create_blob(e.data);
                audio.audio_urls.with_mut(|f: &mut HashMap<BookCursor, String>|f.insert(c.clone(),url));
            }
            audio.guard.set(false);
        }
    }
}

fn create_blob(bytes:Vec<u8>)->String{
    let array = Uint8Array::from(&bytes[..]);
    let parts = Array::new();
    parts.push(&array);
    let bag=BlobPropertyBag::new();
    bag.set_type("audio/mpeg");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts,&bag,).unwrap();
    Url::create_object_url_with_blob(&blob).unwrap()
}



pub fn move_cursor(mut audio: AudioData){
    if (audio.chapter_change)().contains(&(audio.current_cursor)().cursor){
        audio.current_cursor.with_mut(|f|{
            f.cursor.chapter+=1;
            f.cursor.chunk=0;
        });
    }else{
        audio.current_cursor.with_mut(|f|f.cursor.chunk +=1);
    }
}


pub fn switch_audio(audio: AudioData){
    spawn(async move{
        move_cursor(audio.clone());
        get_audio_url(audio).await;
    });
}

pub fn skip_forward(mut audio: AudioData) {
    let current_version = {
        let mut v = (audio.debounce)();
        v += 1;
        audio.debounce.set(v);
        v
    };
    move_cursor(audio.clone());

    spawn(async move {
        sleep(Duration::from_millis(150)).await;

        if (audio.debounce)() == current_version {
            get_audio_url(audio).await;
        }
    });
}

pub fn skip_backward(mut audio: AudioData) {
    audio.current_cursor.with_mut(|c| {
        if c.cursor.chunk > 0 {
            c.cursor.chunk -= 1;
        }
    });

    let current_version = {
        let mut v = (audio.debounce)();
        v += 1;
        audio.debounce.set(v);
        v
    };

    spawn(async move {
        sleep(Duration::from_millis(150)).await;

        if (audio.debounce)() == current_version {
            get_audio_url(audio).await;
        }
    });
}

use wasm_bindgen::JsCast;
pub fn playpause(playing: Signal<bool>){
    let mut playing=playing.clone();
    let document = web_sys::window().unwrap().document().unwrap();
    if let Some(audio) = document.get_element_by_id("my_audio") {
        if *playing.read(){
            let audio: HtmlAudioElement = audio.dyn_into().unwrap();
            playing.set(false);
            let _ = audio.pause();
        }else{
            let audio: HtmlAudioElement = audio.dyn_into().unwrap();
            let _ = audio.play();
        }
    }
}
