use web_sys::{Blob, BlobPropertyBag, Url};
use js_sys::{Array, Uint8Array};
use serde_json;
use std::collections::HashMap;
use dioxus::prelude::*;


use crate::models::{BookStatus, ChunkProgress, GlobalState};


const ADVANCE_AMOUNT: u32 = 10;


pub fn use_audio_chunk_loader(
    time: Signal<f64>,
    audio_url: Signal<Option<String>>,
    chunkmap:Signal<Option<HashMap<String, ChunkProgress>>>
) {
    let global=use_context::<Signal<GlobalState>>();
    let mut time=time.clone();
    let fetch_trigger = use_signal(||0);
    let ended = use_signal(|| false);

    let mut resource = use_resource({
        let global = global.clone();
        let fetch_trigger = fetch_trigger.clone();
        move || {
            let mut global = global.clone();
            let _trigger = fetch_trigger();
            async move { if !ended(){get_audio(&mut global).await} else{None} }
        }
    });
    use_effect({
        let mut audio_url = audio_url.clone();
        let mut chunk=use_signal(||"".to_string());
        let mut signal=use_signal(|| false);
        let mut ready_to_advance = use_signal(|| false);



        move || {
            if audio_url().is_none()  && let Some(Some(bytes)) = resource() {
                audio_url.set(Some(create_blob(bytes.1)));
                signal.set(bytes.0);
                ready_to_advance.set(true);
                match chunkmap() {
                    None=>{},
                    Some(hash)=>{
                        if hash.contains_key(&chunk()){
                            let chunk=hash.get(&chunk()).unwrap();
                            time.set(chunk.start_time);
                        }
                    }
                }

                resource.clear();
            }


            if resource().is_none() && ready_to_advance(){
                let mut fetch_trigger = fetch_trigger.clone();
                let mut global = global.clone();
                let mut ended=false;

                ready_to_advance.set(false);

                global.with_mut(|state| {
                    if let Some(book) = &mut state.book {
                        chunk.set(format!("{},{}", book.chapter as i32, book.chunk as i32));

                        if signal() {
                            if book.chapter < book.max_chapter{
                                book.chunk = 1;
                                book.chapter += 1;
                            }else{
                                ended=true;
                            }
                        } else {
                            book.chunk += ADVANCE_AMOUNT + 1;
                        }
                    }
                });
    
                if !ended {
                    fetch_trigger.set(fetch_trigger() + 1);
                }
            }
        }
    });
}


async fn get_audio(global:&mut Signal<GlobalState>)->Option<(bool,Vec<u8>)>{
    let state=global();
    if state.book.is_none(){return None;}
    
    let book=state.book.clone().unwrap();
    match fetch_audio(book).await {
        Err(_)=>None,
        Ok(vec)=>{Some(vec)
        }
    }
}

async fn fetch_audio(book: BookStatus) -> Result<(bool,Vec<u8>), Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:8000/audio?chunk={}", book.chunk+ADVANCE_AMOUNT);
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
