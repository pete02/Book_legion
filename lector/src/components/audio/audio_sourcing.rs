use dioxus::{logger::tracing, prelude::*};
use js_sys::{Array, Uint8Array};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, Url};

use crate::components::server_api;
use crate::models::{AudioChunkResult, GlobalState, Place};



pub fn audio_sourcing(audio_url: Signal<Option<String>>, jump:Signal<i32>, time:Signal<f64>){
    let audio_url=audio_url.clone();
    let audio_urls: Signal<Vec<(Place,String)>>=use_signal(||Vec::new());
    let reload=use_signal(||true);

    let walk=use_signal(||Place::new(0, 0));
    let resource: Signal<Vec<AudioChunkResult>>=use_signal(||Vec::new());
    
    reload_audio_watcher(reload, resource,audio_url, audio_urls,time);
    audio_url_hook(audio_url, audio_urls, walk);
    audio_urls_hook(audio_urls, resource);
    resource_fetch_hook(resource);
    walker(walk, jump, reload);
}

fn reload_audio_watcher(
    mut reload:Signal<bool>,
    mut resource: Signal<Vec<AudioChunkResult>>,
    mut audio_url: Signal<Option<String>>,
    mut audio_urls:Signal<Vec<(Place,String)>>,
    mut time:Signal<f64>
    ){
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move||{
        if !reload() {return;}
        let Some(book)=global().book.clone() else {return;};

        if reload(){
            resource.set(Vec::new());
            audio_url.set(None);
            audio_urls.set(Vec::new());
            time.set(book.time);
            reload.set(false);
        }

    });
}

fn walker(mut walk:Signal<Place> , mut jump:Signal<i32>, mut reload:Signal<bool>){
    let mut global=use_context::<Signal<GlobalState>>();
    let empty=Place::new(0,0);
    use_effect(move||{
        if jump() !=0{
            global.with_mut(|state|{
                let Some(book)=&mut state.book else {return;};
                let mut cur_place=Place::new(book.chapter, book.chunk);

                if jump()==-1{
                    cur_place.jump_prev(5, &book.chapter_to_chunk);
                }else{
                    cur_place.jump_next(5, &book.chapter_to_chunk);
                }
                book.set_place(cur_place);
                reload.set(true);
            });
            jump.set(0);
            walk.set(empty);
            if reload(){return;}
        }

        if walk()!=empty{
            global.with_mut(|state|{
                let Some(book)=&mut state.book else { return;};
                book.set_place(walk());
                walk.set(empty);
            })
        }
    });

}


fn audio_url_hook(
    mut audio_url: Signal<Option<String>>,
    audio_urls:Signal<Vec<(Place,String)>>,
    mut walk:Signal<Place>
) {
    use_effect(move || {
        if audio_url().is_some() {return; }
        if audio_urls().len() == 0 {return;}
        tracing::debug!("set audio_url");
        let (place,url)=audio_urls().remove(0);
        walk.set(place);
        audio_url.set(Some(url));
    });
}


fn audio_urls_hook(
    audio_urls:Signal<Vec<(Place,String)>>,
    mut resource: Signal<Vec<AudioChunkResult>>
){
    if audio_urls().len() > 0 {return;}
    if resource().len() == 0{return;}
    
    for v in resource(){
        let url=create_blob(v.data);
        let place=Place::parse(&v.place);
        audio_urls().push((place,url));
    }
    tracing::debug!("set audio-urls");
    resource.set(Vec::new());

}

fn resource_fetch_hook(mut resource: Signal<Vec<AudioChunkResult>>){
    let mut fetching=use_signal(||false);
    let global=use_context::<Signal<GlobalState>>();
    let mut last_place=use_signal(||Place::new(0,0));

    use_effect(move ||{
        let Some(mut book)=global().book else {return;};
        let Some(token)=global().access_token else{return;};

        if resource().len() != 0 {return;}
        if fetching() {return;}
        if book.reached_end() {return;}

        let mut fetch_pos;
        if book.get_current_pos() > last_place(){
            fetch_pos=book.get_current_pos()
        }else{
            fetch_pos=last_place();
        }

        book.set_place(fetch_pos.next(&book.chapter_to_chunk));
        fetching.set(true);
        tracing::debug!("start fetch");
        spawn_local(async move{
            match server_api::fetch_audio(&book, token).await{
                Err(a)=>{
                    fetching.set(false);
                    tracing::error!("error in audio fetching: {}",a);
                },
                Ok(vec)=>{
                    tracing::debug!("feched chunks");
                    fetching.set(false);
                    resource.set(vec.clone());
                    let Some(last)=vec.last() else {return;};
                    let place=Place::parse(&last.place);
                    last_place.set(place);
                }
            };
        });

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

