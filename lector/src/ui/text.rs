use dioxus::{logger::tracing, prelude::*};

use crate::{Route, domain::{self, text::use_text}, ui::components::{TopBar, TopBarEntry}};

/// Single component reader
#[component]
pub fn Text(book_id: String) -> Element {
    // Full chapter HTML (Signal)
    let b_signal=use_signal(||book_id.clone());
    let css_ready: Signal<bool>=use_signal(||false);
    let text_handler = use_text(b_signal(), css_ready.clone());
    let mut back=use_signal(||true);


    use_effect(move ||{
        tracing::debug!("here");
        domain::text::fetch_and_apply_book_css(b_signal(),css_ready);
    });


    use_effect(move ||{
        let back=back();
        let vis=text_handler.visible_text.clone();
        if vis().len() >0{
            tracing::debug!("probably updated updated");
            let container=domain::text::get_container();
            tracing::debug!("scroll height: {}", container.scroll_height());
            if back{
                tracing::debug!("back");
                container.set_scroll_top(container.scroll_height());
            }else{
                container.set_scroll_top(0);
            }
        }
    });

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
            TopBar{ entries: top_entries }

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
                    style: "height: 95dvh; overflow: hidden; position: relative;width: 90%;margin-left: 5%; margin-bottom:0%;padding-bottom:10px;",
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
                            let mut val=text_handler.clone();
                            move |_| {
                                back.set(true);
                                domain::page_backwards::render_prev_page(&mut val);
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
                            let mut val= text_handler.clone();
                            move |_|{
                                back.set(false);
                                domain::page_forward::render_next_page(&mut val);
                            }
                        },
                    }
                }
            }
        }
    }
}
