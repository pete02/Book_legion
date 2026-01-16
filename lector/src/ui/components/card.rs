use dioxus::prelude::*;
use crate::{domain::cover::{CardData,use_cover}, styles};

#[component]
pub fn Cover(
    cover_path: Signal<String>,
    #[props(default = "90%".to_string())] width: String,
    #[props(default = "400px".to_string())] max_width: String,
) -> Element {
    let cover = use_cover(cover_path);

    let Some(src) = cover() else {
        return rsx!(div { class: "h-[400px]" });
    };

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
                    cover_path: Signal::new(entry.pic_path.clone()),
                    width: "140px".to_string(),
                    max_width: "160px".to_string(),
                }
                div { style: styles::NAME_STYLE, "{name}" }
            }
        }
    }
}