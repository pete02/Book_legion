use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window};
use crate::models::GlobalState;

pub fn use_css_injector(idle: Signal<bool>, css_idle: Signal<bool>) {
    let mut css_idle=css_idle.clone();
    use_effect(move || {
        if idle() {return;}
        if !css_idle() {return;}

        let global = use_context::<Signal<GlobalState>>();
        let Some(book) = global().book else { return; };

        spawn_local(async move {
            match fetch_css(&book.name).await {
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

async fn fetch_css(book: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:8000/css/{}", book);
    let resp = reqwasm::http::Request::get(&url).send().await?;
    
    if resp.status() >= 500 {
        tracing::error!("Backend error: {:?}", resp.text().await);
    }
    
    let text = resp.text().await?;
    Ok(text)
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
