use dioxus::prelude::*;
use crate::{Route, styles};

#[derive(Clone, Debug, PartialEq)]
pub struct TopBarEntry {
    pub name: String,
    pub path: Route,
}

#[component]
pub fn TopBar(entries: Vec<TopBarEntry>, show_delete: Signal<bool>, on_delete: Option<Callback<()>>) -> Element {
    return rsx! {
        div {
            style: styles::TOPBAR, // we will tweak this to justify space-between

            // Left side: all links grouped
            div {
                style: "display: flex; align-items: center; gap: 12px;",
                {
                    entries.iter().map(|entry| {
                        let path = entry.path.clone();
                        let name = entry.name.clone();
                        rsx!(
                            Link {
                                to: path,
                                button {
                                    class: "
                                        bg-blue-600 hover:bg-blue-700 active:bg-blue-800
                                        text-white font-semibold py-2 px-4
                                        rounded-lg transition-colors duration-150
                                    ",
                                    "{name}"
                                }
                            }
                        )
                    })
                }
            }

            // Right side: delete button (pushed to end)
            if show_delete() {
                if let Some(on_delete) = on_delete {
                    button {
                        class: "
                            bg-red-600 hover:bg-red-700 active:bg-red-800
                            text-white font-semibold py-2 px-4
                            rounded-lg transition-colors duration-150
                        ",
                        onclick: move |_| on_delete.call(()),
                        "Delete"
                    }
                }
            }
        }
    }
}