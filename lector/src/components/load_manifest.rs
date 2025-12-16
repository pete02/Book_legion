use dioxus::{logger::tracing, prelude::*};
use dioxus_signals::Signal;
use wasm_bindgen_futures::spawn_local;

use crate::models::{BookStatus, GlobalState, Manifest};
use crate::components::server_api;

pub fn use_load_manifest(mut manifest: Signal<Vec<BookStatus>> ) {    
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move || {
        tracing::debug!("load manifest");
        let Some(access_token)=global().access_token.clone() else {return;};
        if manifest().len() > 0 {return;};
        spawn_local(async move{
            match server_api::fetch_manifest(access_token).await {
                Err(_)=>tracing::error!("error in loading manifest"),
                Ok(str)=>{
                    match serde_json::from_str::<Manifest>(&str){
                        Err(_)=>tracing::error!("could convert manifest"),
                        Ok(man )=>{
                            let books: Vec<BookStatus> = man
                            .into_iter()
                            .map(BookStatus::from)
                            .collect();
                            manifest.set(books);
                        }
                    }
                }
            }
        });

    
    });
}


