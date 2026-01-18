use std::{collections::{VecDeque, vec_deque}, time::Duration};


use dioxus::{logger::tracing, prelude::*};
use gloo_timers::future::sleep;
use js_sys::{Array, Uint8Array};
use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

use crate::{domain::{self, book::get_book_progress, cursor::{self, BookCursor, Cursor}}, infra};


#[derive(Clone, PartialEq)]
pub struct AudioChunk{
    url:String,
    cursor: domain::cursor::BookCursor
}

#[derive(Clone, PartialEq)]
pub struct AudioData{
    pub book_id:Signal<String>,
    pub name: Signal<String>,
    pub audio_url: Signal<String>,
    pub current_cursor: Signal<BookCursor>,
    pub playing: Signal<bool>,
    pub audio_urls: Signal<Vec<AudioChunk>>,
    pub history: Signal<VecDeque<AudioChunk>>,
    pub progress: Signal<f64>,
    pub audio_guard: Signal<bool>,
    pub debounce_counter: Signal<i32>
}

const MAX_HISTORY: usize = 50;
fn push_to_history(history: &mut Signal<VecDeque<AudioChunk>>, chunk: AudioChunk) {
    let mut local=history();
    local.push_back(chunk);
    if local.len() > MAX_HISTORY {
        local.pop_front(); // remove the oldest chunk
    }

    history.set(local);
}


fn next_audio_url(mut audio: AudioData){
    let mut urls=(audio.audio_urls)();
    if urls.len() > 0{
        let next=urls.remove(0);
        audio.audio_urls.set(urls);
        push_to_history(&mut audio.history, AudioChunk { url: (audio.audio_url)(), cursor: (audio.current_cursor)() });
        audio.audio_url.set(next.url);
        let cursor = next.cursor.clone();
        audio.current_cursor.set(next.cursor);
        spawn(async move {
            let _ = domain::cursor::save_bookcursor(cursor).await;
        });
    }
}

fn prev_audio_url(mut audio: AudioData) {
    let mut history = (audio.history)();
    if let Some(prev_chunk) = history.pop_back() {
        audio.history.set(history);

        let mut urls = (audio.audio_urls)();
        if !(audio.audio_url)().is_empty() {
            urls.insert(
                0,
                AudioChunk {
                    url: (audio.audio_url)(),
                    cursor: (audio.current_cursor)(),
                },
            );
        }
        audio.audio_urls.set(urls);
        audio.audio_url.set(prev_chunk.url.clone());
        audio.current_cursor.set(prev_chunk.cursor.clone());

        let cursor = prev_chunk.cursor.clone();
        spawn(async move {
            let _ = domain::cursor::save_bookcursor(cursor).await;
        });
    }
}

fn error_audio(book_id: String)->AudioData{
    AudioData { 
        book_id: use_signal(||book_id),
        name: use_signal(||"error in getting audio data".to_string()),
        audio_url: use_signal(||"".to_owned()),
        progress: use_signal(||0.0),
        current_cursor: use_signal(||BookCursor::new("u1", "b1", 0, 0)),
        playing: use_signal(||false),
        audio_urls: use_signal(||vec![]),
        history: use_signal(||VecDeque::new()),
        audio_guard: use_signal(||false),
        debounce_counter: use_signal(||0)
    }
}



pub async fn load_audio(book_id:String, mut audio: AudioData){
    let book=domain::book::load_book(book_id.clone()).await;
    let cursor=domain::cursor::load_bookcursor(book_id.clone()).await;
    let mut audio_urls=load_audiourls(cursor.clone()).await;
    if audio_urls.len() > 0{
        let chunk=audio_urls.remove(0);
        audio.audio_url.set(chunk.url);
        domain::cursor::save_bookcursor(chunk.cursor).await;
    }

    audio.book_id.set(book_id.clone());
    audio.name.set(book.title);
    audio.progress.set(domain::book::get_book_progress(book_id.clone()).await);
    audio.playing.set(false);
    audio.audio_urls.set(audio_urls);
    audio.current_cursor.set(cursor);
    refill_audio_buffer(audio.audio_urls.clone(), book_id,audio.audio_guard.clone());
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


pub async fn load_audiourls(cursor:BookCursor)->Vec<AudioChunk>{
    match infra::audio::get_chunks(cursor.clone(), 3).await {
        Err(e)=>{
            tracing::error!("Error in loading audioo_urls: {}",e);
            return vec![]
        },
        Ok(urls)=>{
            let mut audio_urls=vec![];
            for e in urls{
                let mut c=cursor.clone();
                c.cursor=e.cursor;
                let url=create_blob(e.data);
                audio_urls.push(AudioChunk { url: url , cursor:c });
            }

            return audio_urls;
        }
    }
}



pub fn refill_audio_buffer(mut audio_urls:Signal<Vec<AudioChunk>>, book_id: String, mut guard: Signal<bool>){
    if audio_urls().len()< 10 && !guard(){
        guard.set(true);
        spawn(async move{
            let mut cursor=match audio_urls().last() {
                None=>domain::cursor::load_bookcursor(book_id).await,
                Some(chunk)=>chunk.cursor.clone()
            };
            
            cursor.cursor.chunk +=1;

            let mut audios=load_audiourls(cursor).await;
            
            if !audios.is_empty(){
                audio_urls.with_mut(|f|f.append(&mut audios))
            }
            guard.set(false);
        });
    }
}


pub fn switch_audio(audio: AudioData) {
    let mut audio_url = audio.audio_url.clone();
    let mut audio_urls = audio.audio_urls.clone();
    let mut progress = audio.progress.clone();
    let book_id = audio.book_id.clone();
    

    let mut urls = audio_urls();
    if urls.len() >0  {
        let next_chunk=urls.remove(0);

        audio_url.set(next_chunk.url.clone());
        audio_urls.set(urls);

        spawn(async move {
            let _ = domain::cursor::save_bookcursor(next_chunk.cursor).await;
            refill_audio_buffer(audio_urls, book_id(), audio.audio_guard.clone());
            let prog = get_book_progress(book_id()).await;
            progress.set(prog);
        });
    } else {
        handle_empty_audio(audio);
    }
}


pub fn handle_empty_audio(mut audio: AudioData){
    spawn(async move{
        let audioguard=audio.audio_guard.clone();
        let mut audio_urls=audio.audio_urls.clone();
        let book_id=audio.book_id.clone();

        refill_audio_buffer(audio_urls, book_id(), audio.audio_guard);

        while audioguard() {
            sleep(Duration::from_millis(10)).await;
        }

        let mut urls=audio_urls();
        if urls.len() > 0{
            let next=urls.remove(0);
            let _ = domain::cursor::save_bookcursor(next.cursor).await;
            audio_urls.set(urls);
            audio.audio_url.set(next.url);
        }else{
            audio.audio_url.set(String::new());
            audio.playing.set(false);
        }
    });
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

pub fn skip_forward_chunk(mut audio: AudioData) {
    let mut urls = (audio.audio_urls)();

    if urls.len() > 0 {
        let next_chunk=urls.remove(0);
        audio.audio_urls.set(urls);

        audio.audio_url.set(next_chunk.url.clone());
        audio.playing.set(true);

        let current_version = {
            let mut v = (audio.debounce_counter)();
            v += 1;
            audio.debounce_counter.set(v);
            v
        };

        spawn(async move {
            let _ = domain::cursor::save_bookcursor(next_chunk.cursor).await;
            sleep(Duration::from_millis(150)).await;

            if (audio.debounce_counter)()==current_version{
                refill_audio_buffer(audio.audio_urls, (audio.book_id)(), audio.audio_guard);
            }
            let prog = domain::book::get_book_progress((audio.book_id)()).await;
            audio.progress.set(prog);
        });
    } else {
        if (audio.audio_url)()==""{            
            handle_empty_audio(audio)
        }
    }
}

pub fn skip_backward_chunk(mut audio: AudioData) {
    let mut history = (audio.history)();
    if let Some(prev_chunk) = history.pop_back() {
        // Update currently playing URL
        audio.audio_url.set(prev_chunk.url.clone());
        audio.playing.set(true);
        audio.history.set(history);

        // Save cursor asynchronously
        let book_id = audio.book_id.clone();
        spawn(async move {
            let _ = domain::cursor::save_bookcursor(prev_chunk.cursor).await;
            let prog = domain::book::get_book_progress(book_id()).await;
            audio.progress.set(prog);
        });
    } else {
    }
}
