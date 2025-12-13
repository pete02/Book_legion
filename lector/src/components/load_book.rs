use dioxus::{logger::tracing, prelude::*};
use dioxus_signals::Signal;

use crate::models::GlobalState;
use crate::components::server_api;




pub fn use_load_book(mut loaded: Signal<bool>) {
    let mut global = use_context::<Signal<GlobalState>>();
    let mut triggered=use_signal(||false);
    use_effect(move || {
        if triggered() {return;};
        if loaded() {return;}
        let Some(name) = global().name.clone() else {return;};
        let Some(access_token)= global().access_token.clone() else {return;};
        triggered.set(true);

        spawn(async move {
            global.with_mut(|state| state.book = None);
            tracing::info!("loading book: {}",&name);

            match server_api::get_book(name, access_token).await {
                Ok(book) => {
                    global.with_mut(|state| state.book = Some(book));
                    tracing::info!("Book loaded");
                    loaded.set(true);
                    triggered.set(false);
                }
                Err(e) => {
                    tracing::error!("Error in book loading: {}",e);
                    triggered.set(false);
                }
            }
        });
    });
}


