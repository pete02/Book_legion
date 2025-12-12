use dioxus::{prelude::*};


use crate::components::{book::*, global_updater, use_load_book};

#[component]
pub fn BookView()->Element{
    
    let book=use_signal(||"fused".to_owned());
    let html_vec: Signal<Vec<String>> = use_signal(Vec::new);
    let visible_chunks=use_signal(||Vec::new());
    let move_page=use_signal(||0);

    use_load_book(book());
    use_css_injector();
    chapter_fetch_hook(html_vec);
    page_navigator(move_page, html_vec, visible_chunks);

    global_updater();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

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