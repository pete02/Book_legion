use dioxus::prelude::*;

use crate::{components::{BookCover, use_load_manifest}, models::{BookStatus, GlobalState}, views::Route};

// Main Library View Component
#[component]
pub fn LibraryView() -> Element {
    let manifest: Signal<Vec<BookStatus>> = use_signal(||Vec::new());
    let mut global=use_context::<Signal<GlobalState>>();
    // Load the manifest on first render
    use_load_manifest(manifest.clone());
    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                padding: 24px;
                overflow-y: auto;
                height: 100%;
                box-sizing: border-box;
            ",

            h1 {
                style: "margin-bottom: 16px;",
                "Library"
            }

            // Responsive grid
            div {
                style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
                    gap: 24px;
                    width: 100%;
                ",
                {
                manifest.iter().map(|b| {
                    let name = b.name.clone();
                    let onclick_name = name.clone();
                    let display_name = name.clone();

                    rsx! {
                        // Each book card
                        Link {
                            to: Route::BookView {},
                            onclick: move |_| {
                                global.with_mut(|state|{
                                    state.name=Some(onclick_name.clone())
                                });
                            },

                            div {
                                style: "
                                    display: flex;
                                    flex-direction: column;
                                    align-items: center;
                                    padding: 12px;
                                    border-radius: 12px;
                                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                                    cursor: pointer;
                                    transition: transform 0.15s ease-in-out;
                                ",

                                // Book cover
                                BookCover {
                                    name: Signal::new(b.name.clone()),
                                    width: "140px".to_string(),
                                    max_width: "160px".to_string(),
                                }

                                // Name beneath the cover
                                div {
                                    style: "margin-top: 8px; text-align: center; font-weight: bold;",
                                    "{display_name}"
                                }
                            }
                        }
                    }
                })
                }
            }
        }
    }
}
