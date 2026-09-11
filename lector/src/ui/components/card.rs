use dioxus::{logger::tracing, prelude::*};
use crate::{domain::{cover::{CardData,get_cached_cover}, login}, styles};

#[component]
pub fn Cover(
    book_id: String,
    #[props(default = "90%".to_string())] width: String,
    #[props(default = "400px".to_string())] max_width: String,
) -> Element {
    let cover = use_resource({
        let book_id = book_id.clone();
        let width = width.clone();
        move || {
            let book_id = book_id.clone();
            let width = width.clone();
            async move {
                match get_cached_cover(&book_id, &width).await {
                    Ok(url) => {
                        tracing::debug!("Cover URL for book {}: {}", book_id, url);
                        url
                    }
                    Err(_) => {
                        tracing::error!("Failed to fetch cover for book: {}", book_id);
                        String::new()
                    }
                }
            }
        }
    });

    let src = cover.read().clone().unwrap_or_default();

    rsx! {
        img {
            class: "rounded-xl shadow-md object-contain",
            style: "width: {width}; max-width: {max_width}; height: auto; margin: 16px; display: block",
            src: "{src}"
        }
    }
}
#[component]
pub fn Card(entry: CardData) -> Element {
    let name = entry.name.clone();
    rsx! {
        Link { to:  entry.path,
            div { style: styles::CARD_STYLE,
                Cover {
                    book_id: entry.pic_path.clone(),
                    width: "140px".to_string(),
                    max_width: "160px".to_string(),
                }
                div { style: styles::NAME_STYLE, "{name}" }
            }
        }
    }
}