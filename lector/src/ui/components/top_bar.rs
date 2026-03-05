use dioxus::prelude::*;
use crate::{Route, styles};

#[derive(Clone, Debug, PartialEq)]
pub struct TopBarEntry {
    pub name: String,
    pub path: Route,
}

#[component]
pub fn TopBar(
    entries: Vec<TopBarEntry>,
    show_extra: Signal<bool>,
    text_extra: Option<String>,
    on_extra: Option<Callback<()>>,
) -> Element {
    rsx! {
        div {
            style: styles::TOPBAR,
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                { entries.iter().map(|entry| {
                    let path = entry.path.clone();
                    let name = entry.name.clone();
                    rsx!(
                        Link {
                            to: path,
                            button {
                                class: "bg-blue-600 hover:bg-blue-700 active:bg-blue-800
                                        text-white font-semibold py-2 px-4
                                        rounded-lg transition-colors duration-150",
                                "{name}"
                            }
                        }
                    )
                })}
            }
            if show_extra() {
                if let Some(on_extra) = on_extra && let Some(text) = text_extra {
                    button {
                        class: "bg-red-600 hover:bg-red-700 active:bg-red-800
                                text-white font-semibold py-2 px-4
                                rounded-lg transition-colors duration-150",
                        onclick: move |_| on_extra.call(()),
                        {text}
                    }
                }
            }
        }
    }
}