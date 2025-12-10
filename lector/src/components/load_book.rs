use dioxus::{logger::tracing, prelude::*};
use dioxus_signals::Signal;

use crate::models::GlobalState;
use crate::components::server_api;


#[derive(Clone, PartialEq)]
enum LoadStatus {
    Loading,
    Success,
    Error(String),
}


pub fn use_load_book(book_name:String, time: Signal<f64>, idle:Signal<bool>) {
    let global = use_context::<Signal<GlobalState>>();
    let status = use_signal(|| LoadStatus::Loading);
    let mut idle=idle.clone();
    let mut time=time.clone();
    let mut loaded=use_signal(||false);
    
    use_effect(move || {
        if loaded() {return;}
        let mut global = global.clone();
        let mut status = status.clone();
        let value = book_name.clone();

        spawn(async move {
            global.with_mut(|state| state.book = None);
            tracing::info!("loading book: {}",&value);

            match server_api::get_book(value).await {
                Ok(book) => {
                    time.set(book.time.clone());
                    global.with_mut(|state| state.book = Some(book));
                    tracing::info!("Book loaded");
                    status.set(LoadStatus::Success);
                    idle.set(false);
                    loaded.set(true);
                }
                Err(e) => {
                    tracing::error!("Error in book loading: {}",e);
                    status.set(LoadStatus::Error(format!("{}", e)));
                }
            }
        });
    });
}


