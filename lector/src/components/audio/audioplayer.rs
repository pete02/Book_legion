use dioxus::prelude::*;
use dioxus_signals::Signal;
use gloo_timers::future::TimeoutFuture;
use web_sys::{Blob, BlobPropertyBag, Url};
use js_sys::{Array, Uint8Array};
use serde_json;
use wasm_bindgen_futures::spawn_local;


use crate::models::{GlobalState, BookStatus};


const ADVANCE_AMOUNT: u32 = 10;
const TICK_INTERVAL_MS: u32 = 100;
const TICK_INCREMENT: f64 = TICK_INTERVAL_MS as f64 /1000.0;

#[component]
pub fn AudioPlayer(playing: Signal<bool>, total_played: Signal<f64>) -> Element {
    let globalstate = use_context::<Signal<GlobalState>>();
    let audio_url = use_signal(|| None::<String>);
    let fetch_trigger = use_signal(|| 0u32);



    use_audio_chunk_loader(globalstate.clone(), audio_url.clone(), fetch_trigger.clone());
    use_playback_tick(playing.clone(), total_played.clone());

    if let Some(src) = audio_url() {
        render_audio(&src, playing.clone(), audio_url.clone())
    } else {
        rsx!(div { id: "audio-player" })
    }
}

fn use_audio_chunk_loader(
    global: Signal<GlobalState>,
    audio_url: Signal<Option<String>>,
    fetch_trigger: Signal<u32>,
) {
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
        let mut global = global.clone();
        let mut audio_url = audio_url.clone();
        let mut fetch_trigger = fetch_trigger.clone();
        let mut ended=ended.clone();
        move || {
            if audio_url().is_none()  && let Some(Some(bytes)) = resource() {
                audio_url.set(Some(create_blob(bytes.1)));
                resource.clear();
                global.with_mut(|state| {
                    if let Some(book) = &mut state.book {
                        if bytes.0 {
                            if book.chapter < book.max_chapter{
                                book.chunk = 1;
                                book.chapter += 1;
                            }else{
                                ended.with_mut(|f|*f=true);
                            }
                        } else {
                            book.chunk += ADVANCE_AMOUNT + 1;
                        }
                    }
                });
    
                if !ended() {
                    fetch_trigger.set(fetch_trigger() + 1);
                }
            }
        }
    });
}


fn use_playback_tick(playing: Signal<bool>, mut total_played: Signal<f64>) {
    use_effect(move || {
        spawn_local(async move {
            loop {
                TimeoutFuture::new(TICK_INTERVAL_MS).await;
                if *playing.read() {
                    total_played.with_mut(|t| *t += TICK_INCREMENT);
                }
            }
        });
    });
}


fn render_audio(src: &str, mut playing: Signal<bool>, mut audio_url: Signal<Option<String>>) -> Element {
    rsx! {
        div { id: "audio-player",
            audio {
                id: "my_audio",
                controls: true,
                style: "display:none",
                autoplay: true,
                src: "{src}",
                onplay: move |_| playing.set(true),
                onpause: move |_| playing.set(false),
                onended: move |_| {
                    playing.set(false);
                    audio_url.set(None);
                },
            }
        }
    }
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
