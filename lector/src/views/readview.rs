use dioxus::{prelude::*};


use crate::{components::{book::*, global_updater, load_name}, models::GlobalState, views::Route};

#[component]
pub fn ReadView()->Element{
    let global = use_context::<Signal<GlobalState>>();
    let navigator = use_navigator();

    let ok=global().book.is_some() && 
        global().access_token.is_some() &&
        global().refresh_token.is_some();


    if !ok{
        use_effect(move ||{
            navigator.replace(Route::BookView {  });
        });
        return rsx!(div {});
    }

    ReadInner()
}


#[component]
pub fn ReadInner()->Element{
    
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
            style: "
                display: flex;
                flex-direction: column;
                flex: 1 1 auto;
                min-height: 0;
                overflow: visible;
            ",
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

            div {
                style: "
                    position: relative;
                    display: flex;
                    flex-direction: column;
                    overflow: visible;
                    flex: 1 1 auto;
                    min-height: 90vh;
                ",
                id: "text-container",
                BookRenderer { visible_chunks }
                BookButtons { move_page }
            }
        }
    }
 }