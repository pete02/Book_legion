use dioxus::prelude::*;
use crate::{domain::cover::{CardData}, styles};

#[component]
pub fn Cover(
    cover_path: String,
    #[props(default = "90%".to_string())] width: String,
    #[props(default = "400px".to_string())] max_width: String,
) -> Element {
    rsx! {
        img {
            class: "rounded-xl shadow-md object-contain",
            style: "width: {width}; max-width: {max_width}; height: auto; margin: 16px; display: block",
            src: "{cover_path}"
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