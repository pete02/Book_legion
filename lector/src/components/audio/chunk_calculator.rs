use std::{collections::HashMap};
use gloo_timers::future::TimeoutFuture;

use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::models::{BookStatus, ChunkData, ChunkProgress,  GlobalState};


pub fn use_chunk_calculator(time: Signal<f64>, chunkmap: Signal<Option<HashMap<String,ChunkProgress>>>){
    let time= time.clone();
    let global = use_context::<Signal<GlobalState>>();
    let chunks = use_signal(|| Vec::<ChunkProgress>::new());


    let _ = global().book.clone();
    use_effect(move || {
        if global().book.is_none() {
            return;
        }

        if !chunks().is_empty() {
            return;
        }
        spawn_local(async move {
            if global().book.is_some() {
                load_audiomap(chunks, global, chunkmap).await;
            }
        });
    });
    check_chunk(&time, &chunks);

}

fn check_chunk(time: &Signal<f64>, chunks: &Signal<Vec<ChunkProgress>>) {
    let time = time.clone();
    let chunks = chunks.clone();
    let global=use_context::<Signal<GlobalState>>();
    
    let mut cur_chunk=use_signal(||ChunkProgress {
        chapter_number: 0,
        chunk_number: 0,
        start_time: 0.0,
        duration: 0.0
    });

    use_effect(move || {
        let running = std::rc::Rc::new(std::cell::Cell::new(true));
        let running_ref = running.clone();

        spawn_local(async move {
            while running_ref.get() {
                TimeoutFuture::new(1000).await;
                let list = chunks();

                if list.is_empty() {
                    tracing::debug!("no chunks loaded yet");
                    continue;
                }
                let idx = list.partition_point(|c| c.start_time <= time());

                let active = if idx == 0 { 0} else { idx - 1 };

                let chunk = &list[active];
                if *chunk != cur_chunk(){
                    cur_chunk.set(chunk.clone());
                    match global().book {
                        None=>{},
                        Some(b)=>{let _=update_progress(chunk.clone(), b).await;}
                    }
                }
            }
        });
    });
}




async fn load_audiomap(chunks: Signal<Vec::<ChunkProgress>>,global:Signal<GlobalState>, mut chunkmap: Signal<Option<HashMap<String,ChunkProgress>>>){
    tracing::debug!("loading map");
    let mut chunks=chunks.clone();
    let mut res=chunks();
    match global().book {
        None=>{},
        Some(book)=>{
            match fetch_audiomap(book).await {
                Err(e)=>tracing::error!("could not fetch audiomap: {e}"),
                Ok(hash)=>{
                    chunkmap.set(Some(hash.clone()));
                    res = hash.values().cloned().collect::<Vec<_>>();
                    res.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap_or(std::cmp::Ordering::Equal));
                }
            }
        }
    }
    chunks.set(res);
}

async fn fetch_audiomap(book: BookStatus) -> Result<HashMap<String,ChunkProgress>, Box<dyn std::error::Error>> {
    let data=reqwasm::http::Request::post("http://127.0.0.1:8000/audiomap")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&book)?)
        .send()
        .await?.text().await?;


    let chunkdata:ChunkData=serde_json::from_str(&data)?;
    let map: HashMap<String, ChunkProgress>=chunkdata.data.map;
    return Ok(map)
}

async fn update_progress(chunk:ChunkProgress,mut book:BookStatus)->Result<(),Box<dyn std::error::Error>>{
    book.chapter=chunk.chapter_number;
    book.chunk=chunk.chunk_number;
    book.time=chunk.start_time;

    let _=reqwasm::http::Request::post("http://127.0.0.1:8000/update")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&book)?)
        .send()
        .await?.text().await?;

    Ok(())
}