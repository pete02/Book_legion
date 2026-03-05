use dioxus::{logger::tracing, prelude::*};


use crate::{Route, domain::{self, text_renderer::{Align, render, use_renderer}}, ui::components::{TopBar, TopBarEntry}};

/// Single component reader
#[component]
pub fn Text(book_id: String) -> Element {
    // Full chapter HTML (Signal)
    let b_signal=use_signal(||book_id.clone());
    let css_ready: Signal<bool>=use_signal(||false);
    let mut align=use_signal(||Align::None);
    let mut text_handler = use_renderer(book_id.clone(), align.clone());
    
    let show_extra = use_signal(|| false);  


    use_effect(move ||{
        tracing::debug!("here");
        domain::text::fetch_and_apply_book_css(b_signal(),css_ready);
    });


    render(&text_handler, align, book_id.clone());
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
    if !css_ready() {
        return rsx!(div {id: "book-renderer", "Loading reader…" });
    }

    return rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                flex: 1 1 auto;
                min-height: 0;
                overflow: hidden;
            ",
            TopBar{ entries: top_entries, show_extra: show_extra }

            div {
                style: "
                    position: relative;
                    display: flex;
                    flex-direction: column;
                    overflow: hidden;
                    flex: 1 1 auto;
                    min-height: 95dvh;
                ",
                id: "text-container",

                // Paged chapter display
                div {
                    id: "book-renderer",
                    style: "height: 90dvh; overflow: hidden; position: relative;width: 90%;margin-left: 5%; margin-bottom:0%;padding-bottom:10px; text-align: left !important;",
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
                        onclick: {
                            move |_| {
                                align.set(Align::Bottom);
                            }
                    },
                    },
                    // Right half
                    button {
                        style: "
                            flex: 1 1 0;
                            cursor: pointer;
                            background: transparent;
                        ",
                        onclick: {
                            move |_|{
       
                                text_handler.start_offset.set((text_handler.end_offset)());
                                
                                tracing::debug!("t start: {}", (text_handler.start_offset)());
                                align.set(Align::Top);
                            }
                        },
                    }
                }
            }
        }
    }
}
