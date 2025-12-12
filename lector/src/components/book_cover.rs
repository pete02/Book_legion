use dioxus::prelude::*;

#[component]
pub fn BookCover(
    name: Signal<String>,
    #[props(default = "90%".to_string())] width: String,
    #[props(default = "400px".to_string())] max_width: String,
) -> Element {
    if name().is_empty() {
        return rsx!(Fragment {});
    }

    rsx! {
        img {
            class: "rounded-xl shadow-md object-contain",
            style: "width: {width}; max-width: {max_width}; height: auto; margin: 16px; display: block",
            src: "http://127.0.0.1:8000/cover/{name}"
        }
    }
}