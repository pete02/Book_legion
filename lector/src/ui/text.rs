use dioxus::prelude::*;

use crate::{Route, domain::{self, text::use_text}, ui::components::{TopBar, TopBarEntry}};

/// Single component reader
#[component]
pub fn Text(book_id: String) -> Element {
    // Full chapter HTML (Signal)
    let mut text_handler = use_text(book_id.clone());

    // Page index and move delta
    let mut move_page = use_signal(|| 0);

    

    // Top bar entries
    let top_entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {  }
        },
        TopBarEntry {
            name: "Book".into(),
            path: Route::Book { book_id: book_id.clone() }
        },
    ];

    return rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                flex: 1 1 auto;
                min-height: 0;
                overflow: visible;
            ",
            TopBar{ entries: top_entries }

            div {
                style: "
                    position: relative;
                    display: flex;
                    flex-direction: column;
                    overflow: visible;
                    flex: 1 1 auto;
                    min-height: 95vh;
                ",
                id: "text-container",

                // Paged chapter display
                div {
                    id: "book-renderer",
                    style: "height: 95vh; overflow: hidden; position: relative;width: 90%;margin-left: 5%; margin-bottom:0%",
                    dangerous_inner_html: "{text_handler.visible_text}"
                }

                div {
                    style: "
                        position: absolute;
                        top: 0;
                        left: 0;
                        width: 100%;
                        height: 100%;
                        display: flex;
                    ",
                    // Left half
                    button {
                        style: "
                            flex: 1 1 0;
                            cursor: pointer;
                            background: transparent;
                        ",
                        onclick: move |_| { move_page.set(-1); },
                    },
                    // Right half
                    button {
                        style: "
                            flex: 1 1 0;
                            cursor: pointer;
                            background: transparent;
                        ",
                        onclick: move |_| { domain::page_forward::render_next_page(&mut text_handler);},
                    }
                }
            }
        }
    }
}
