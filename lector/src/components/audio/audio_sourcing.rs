

use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;
use js_sys::{Array, Uint8Array};
use web_sys::{Blob, BlobPropertyBag, Url};

use crate::components::server_api;
use crate::models::{AudioChunkResult, BookStatus, GlobalState};
use crate::components::audio::ADVANCE_AMOUNT;



pub fn audio_sourcing(audio_url: Signal<Option<String>>, reload:Signal<bool>, time:Signal<f64>){
    let audio_url=audio_url.clone();
    let audio_urls: Signal<Vec<(String,String)>>=use_signal(||Vec::new());
    let resource: Signal<Option<Vec<AudioChunkResult>>>=use_signal(||None);
    let private_state=use_signal(||None);
    
    reload_audio_watcher(private_state,reload, resource,audio_url, time);
    audio_url_hook(audio_url, audio_urls,resource);
    resource_fetch_hook(resource, private_state);
}

fn reload_audio_watcher(
    mut private_state:Signal<Option<BookStatus>>,
    mut reload:Signal<bool>,
    mut resource: Signal<Option<Vec<AudioChunkResult>>>,
    mut audio_url: Signal<Option<String>>,
    mut time:Signal<f64>
    ){
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move||{
        if !reload() {return;}
        let Some(book)=global().book.clone() else {return;};

        private_state.set(Some(book.clone()));
        if reload(){
            resource.set(None);
            audio_url.set(None);
            time.set(book.time);
            reload.set(false);
        }

    });
}


fn audio_url_hook(
    mut audio_url: Signal<Option<String>>,
    mut audio_urls:Signal<Vec<(String,String)>>,
    mut resource: Signal<Option<Vec<AudioChunkResult>>>,
) {
    use_effect(move || {
        if audio_url().is_some() {return; }
        if audio_urls().len()>0{
            audio_urls.with_mut(|vec|{
                let (place, url)=vec.remove(0);
                audio_url.set(Some(url));

                
                tracing::debug!("load chunk: {}", place)
            });
            tracing::debug!("urls len: {}", audio_urls.len());
        }else{
            let Some(vec)=resource() else {return;};
            let mut urls=Vec::new();
            for v in vec {
                let url=create_blob(v.data);
                urls.push((v.place, url));
            }
            tracing::debug!("audio urls len: {}", urls.len());
            audio_urls.set(urls);
            resource.set(None);
        }
    });
}

fn resource_fetch_hook(mut resource: Signal<Option<Vec<AudioChunkResult>>>, mut private_state:Signal<Option<BookStatus>>){
    let mut fetching=use_signal(||false);
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move ||{
        if fetching() {return;}
        if resource().is_some() {return;}
        let Some(mut book)=private_state().clone() else {return;};
        let Some(access_token)= global().access_token.clone() else {return;};
        if book.chapter > book.max_chapter {return;}

        fetching.set(true);
        spawn_local(async move{
            match server_api::fetch_audio(&book, access_token).await{
                Err(e)=>{
                    tracing::error!("error in audio_fetch: {e}");
                    fetching.set(false);
                }
                Ok(vec)=>{
                    if vec[vec.len()-1].reached_end{
                        book.chapter+=1;
                        book.chunk=1
                    }else{
                        book.chunk +=ADVANCE_AMOUNT;
                    }

                    tracing::debug!("current book chunk: {}", book.chunk);
                    private_state.set(Some(book));
                    tracing::debug!("resource len: {}", vec.len());
                    resource.set(Some(vec));
                    fetching.set(false);
                }
            }
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
