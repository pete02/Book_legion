use dioxus::{hooks::{use_context, use_effect}, signals::Signal};
use wasm_bindgen_futures::spawn_local;

use crate::models::{BookStatus, GlobalState};


pub fn global_watcher(){
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move ||{
        
        let Some(book)=global().book else {return;};

        spawn_local(async move{
            let _ =update_progress(book).await;
        });
    });
}

async fn update_progress(book:BookStatus)->Result<(),Box<dyn std::error::Error>>{
    let _=reqwasm::http::Request::post("http://127.0.0.1:8000/update")
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&book)?)
        .send()
        .await?.text().await?;

    Ok(())
}