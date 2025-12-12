use std::{collections::HashMap};

use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::models::{BookStatus, ChunkProgress, GlobalState};
use crate::components::server_api;



pub fn use_chunk_calculator(time: Signal<f64>, reload:Signal<bool>){
    let time= time.clone();
    let chunks = use_signal(|| Vec::<ChunkProgress>::new());

    chunk_hook(chunks);
    check_chunk_hook(&time, &chunks, reload);
}


fn chunk_hook(mut chunks: Signal<Vec::<ChunkProgress>>){
    let chunkmap: Signal<Option<(String, HashMap<String,ChunkProgress>)>>=use_signal(||None);
    let global = use_context::<Signal<GlobalState>>();
    let map_fetching=use_signal(||false);
    use_effect(move || {
        let Some(book)=global().book.clone() else {return;};
        if map_fetching() { return;}


        let Some((name, _))= chunkmap() else{
            chunks.set(Vec::new());
            load_chunkmap(book, chunkmap, map_fetching);
            return;
        };

        if name !=book.name{
            chunks.set(Vec::new());
            load_chunkmap(book, chunkmap,map_fetching);
            return;
        }

        if chunks.len()==0 && name==book.name{
            load_audiomap(chunks,  chunkmap)
        }

    });
}

fn load_audiomap(chunks: Signal<Vec::<ChunkProgress>>, chunkmap: Signal<Option<(String, HashMap<String,ChunkProgress>)>>){
    let mut chunks=chunks.clone();
    let Some((_,map))=chunkmap()else {return;};

    let mut res = map.values().cloned().collect::<Vec<_>>();
    res.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap_or(std::cmp::Ordering::Equal));
    chunks.set(res);
}

fn load_chunkmap(book:BookStatus, mut chunkmap: Signal<Option<(String, HashMap<String,ChunkProgress>)>>, mut map_fetching: Signal<bool>){
    map_fetching.set(true);
    spawn_local(async move {
        let Ok(map)=server_api::fetch_audiomap(&book).await else {map_fetching.set(false); return;};
        chunkmap.set(Some((book.name, map)));
        map_fetching.set(false);
    });
}


fn check_chunk_hook(time: &Signal<f64>, chunks: &Signal<Vec<ChunkProgress>>, reload:Signal<bool>) {
    let time = time.clone();
    let chunks = chunks.clone();
    
    use_effect(move || {
        let time=time();
        let chunks=chunks();
        if chunks.is_empty() {
            tracing::debug!("no chunks loaded yet");
            return;
        }
        let chunk=&derive_chunk(time, chunks);
        update_global_state(chunk,reload);
    });
}

fn derive_chunk(time:f64,chunks:Vec<ChunkProgress>)->ChunkProgress{
    let idx: usize = chunks.partition_point(|c| c.start_time <= time);
    let active = if idx == 0 { 0} else { idx - 1 };
    chunks[active].clone()
}


fn update_global_state(chunk: &ChunkProgress, mut reload:Signal<bool>){
    let mut global=use_context::<Signal<GlobalState>>();
    global.with_mut(|state|{
        let Some(book)=&mut state.book else {return;};
        if book.chunk == chunk.chunk_number && book.chapter==chunk.chapter_number {return;}

        if book.chunk != chunk.chunk_number-1||
        (chunk.chunk_number == 1 && book.chapter != chunk.chapter_number-1){
            reload.set(true);
        }

        book.chapter=chunk.chapter_number;
        book.chunk=chunk.chunk_number;
        book.time=chunk.start_time;
    })
}