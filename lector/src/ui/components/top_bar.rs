use dioxus::prelude::*;
use crate::{Route, styles};

#[derive(Clone, Debug, PartialEq)]
pub struct TopBarEntry {
    pub name: String,
    pub path: Route,
}

#[component]
pub fn TopBar(entries: Vec<TopBarEntry>) -> Element {
    return rsx! {
        div {
            style: styles::TOPBAR,
            {
            entries.iter().map(|entry| {
                let path = entry.path.clone();
                let name=entry.name.clone();
                rsx!(
                    Link{
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
    }
}