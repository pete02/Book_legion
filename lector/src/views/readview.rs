use dioxus::{prelude::*};


use crate::{components::{book::*, global_updater, load_name}, views::Route};

#[component]
pub fn ReadView()->Element{
    
    let book=use_signal(||"".to_owned());
    let html_vec: Signal<Vec<String>> = use_signal(Vec::new);
    let visible_chunks=use_signal(||Vec::new());
    let move_page=use_signal(||0);

    use_css_injector();
    load_name(book);
    chapter_fetch_hook(html_vec);
    page_navigator(move_page, html_vec, visible_chunks);

    global_updater();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            div {
                style: "display: flex; justify-content: flex-start; gap: 12px; align-items: center; padding: 8px 16px;",
                class: "bg-gray-200 dark:bg-gray-800 px-4", 

                Link {
                    to: Route::LibraryView {  },
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Library"
                    }
                }
                Link{
                    to: Route::BookView {},
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Book"
                    }
                }

                Link{
                    to: Route::AudioView {},
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Listen"
                    }
                }
            }

            h1 {
                style: "flex: 0 0 auto;",
                "{book}",
            },

            div {
                style: "
                    position: relative;
                    flex: 1 1 0;
                    display: flex;
                    flex-direction: column;
                    overflow: hidden;
                ",
                BookRenderer { visible_chunks }
                BookButtons { move_page }
            }
        }
    }
 }