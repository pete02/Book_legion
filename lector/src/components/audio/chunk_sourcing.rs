use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, Url};
use js_sys::{Array, Uint8Array};
use serde_json;
use dioxus::prelude::*;


use crate::models::{BookStatus, GlobalState};


const ADVANCE_AMOUNT: u32 = 10;

pub fn use_audio_chunk_loader(audio_url: Signal<Option<String>>, idle:Signal<bool>){
    let audio_url=audio_url.clone();
    let resource=use_signal(||None);
    let end=use_signal(||false);
    let stop_fetch=use_signal(||false);

    audio_url_hook(audio_url, resource, idle);
    fetch_for_resource(resource, end, stop_fetch,idle);
    advance_book_hook(resource, end, stop_fetch, idle);
}



fn audio_url_hook(
    audio_url: Signal<Option<String>>,
    resource: Signal<Option<Vec<u8>>>,
    idle:Signal<bool>
) {
    let mut audio_url = audio_url.clone();
    let mut resource = resource.clone();

    use_effect(move || {
        if audio_url().is_some() {return; }
        if idle() {return;}

        let Some(bytes) = resource() else {
            return;
        };

        let url = create_blob(bytes);
        audio_url.set(Some(url));
        resource.set(None);
    });
}

fn fetch_for_resource(resource: Signal<Option<Vec<u8>>>, end: Signal<bool>, stop_fetch:Signal<bool>, idle:Signal<bool>){
    let resource=resource.clone();
    let mut fetching=use_signal(||false);
    let end = end.clone();
    let mut stop_fetch=stop_fetch.clone();
    let global=use_context::<Signal<GlobalState>>();

    use_effect(move ||{
        if fetching() {return;}
        if resource().is_some() {return;}
        if stop_fetch() {return;}
        if idle() {return;}

        let Some(book)=global().book else {return;};

        
        fetching.set(true);
        let mut resource = resource.clone();
        let mut end = end.clone();
        let mut fetching = fetching.clone();

        spawn_local(async move {
            match fetch_audio(book).await{
                Err(_)=>{fetching.set(false);},
                Ok((reached_end,bytes))=>{
                    end.set(reached_end);
                    resource.set(Some(bytes));
                    fetching.set(false);
                    stop_fetch.set(true);
                }
            }
        });
    });
}


fn advance_book_hook(resource: Signal<Option<Vec<u8>>>, end: Signal<bool>, stop_fetch:Signal<bool>, idle:Signal<bool>){
    let mut global = use_context::<Signal<GlobalState>>();
    let end=end.clone();
    let resource=resource.clone();
    let mut stop_fetch=stop_fetch.clone();

    use_effect(move || {
        if resource().is_none() {return;}
        if idle() {return;}

        global.with_mut(|state|{
            if let Some(book)=&mut state.book {
                if !end(){
                    book.chunk+= ADVANCE_AMOUNT+1;
                    stop_fetch.set(false);
                }else{
                    if book.chapter < book.max_chapter{
                        book.chapter +=1;
                        book.chunk=1;
                        stop_fetch.set(false);
                    }
                }
            }
        })
    });
}

async fn fetch_audio(book: BookStatus) -> Result<(bool,Vec<u8>), Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:8000/audio?chunk={}", ADVANCE_AMOUNT);
    let bytes = reqwasm::http::Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&book)?)
        .send()
        .await?;
    match bytes.headers().get("reached-end"){
        Some(value)=> Ok((value == "true", bytes.binary().await?)),
        None=>return Err("incorrect headers".into())
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
