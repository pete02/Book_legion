use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window};
use crate::models::GlobalState;
use crate::components::server_api;

pub fn use_css_injector() {
    let mut css_idle=use_signal(||true);
    let global = use_context::<Signal<GlobalState>>();
    use_effect(move || {
        if !css_idle() {return;}
        let Some(book) = global().book.clone() else { return; };

        spawn_local(async move {
            match server_api::fetch_css(&book.name).await {
                Ok(css_text) => {
                    inject_or_append_css("book-css", &css_text);
                    css_idle.set(false);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch CSS: {:?}", e);
                    css_idle.set(false);
                }
            }
        });
    });
}



fn inject_or_append_css(id: &str, css_text: &str) {
    let document = window().unwrap().document().unwrap();

    let style_el = document
        .get_element_by_id(id)
        .unwrap_or_else(|| {
            let el = document.create_element("style").unwrap();
            el.set_id(id);
            document.head().unwrap().append_child(&el).unwrap();
            el
        });

    let current_css = style_el.inner_html();
    style_el.set_inner_html(&format!("{current_css}\n{css_text}"));
}
