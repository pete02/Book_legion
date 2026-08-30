use dioxus::{logger::tracing, prelude::*};
use crate::{domain::{cover::{CardData,get_cached_cover}, login}, styles};

#[component]
pub fn Cover(
    book_id: String,
    #[props(default = "90%".to_string())] width: String,
    #[props(default = "400px".to_string())] max_width: String,
) -> Element {
    let mut link: Signal<String> = use_signal(||"".to_owned());
    use_effect(move ||{
        let b=book_id.clone();
        spawn(async move{
            let blob=get_cached_cover(&b).await;
            match blob{
                Ok(url) => {
                    tracing::debug!("Cover URL for book{}", url);
                    link.set(url)
                },
                Err(_) => {
                    tracing::error!("Failed to fetch cover for book: {}", b);
                    link.set(String::new());
                }
            }
        });
    });
    rsx! {
        img {
            class: "rounded-xl shadow-md object-contain",
            style: "width: {width}; max-width: {max_width}; height: auto; margin: 16px; display: block",
            src: "{link}"
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