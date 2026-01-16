use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::{components::server_api, models::GlobalState};

pub fn chapter_fetch_hook(mut html_vec: Signal<Vec<String>>){
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move || {
        let Some(book) = global().book.clone() else { return };
        if html_vec().len() > 0 {return;};
        let Some(access_token)= global().access_token.clone() else {return;};
        spawn_local(async move {
            match server_api::fetch_chapter(book,access_token).await {
                Ok(chapter) => {
                    let vec = strip_headers(&chapter)
                        .split("\n")
                        .map(|s| s.to_string())
                        .filter(|s| !s.trim().is_empty())
                        .collect::<Vec<_>>();
                    html_vec.set(vec);
                }
                Err(e) => tracing::error!(?e, "Error fetching chapter"),
            }
        });
    });
}

use regex::Regex;

fn strip_headers(xhtml: &str) -> String {
    let re = Regex::new(
        r"(?s)^<\?xml[^>]*>\s*<html[^>]*>.*?<body[^>]*>\s*|\s*</body>\s*</html>\s*$",
    )
    .unwrap();
    re.replace_all(xhtml, "").to_string()
}