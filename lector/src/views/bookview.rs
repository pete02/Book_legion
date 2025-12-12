use dioxus::{prelude::*};


use crate::components::{book::*, global_updater, use_load_book};

#[component]
pub fn BookView()->Element{
    
    let book=use_signal(||"fusing".to_owned());
    let html_vec: Signal<Vec<String>> = use_signal(Vec::new);
    let visible_chunks=use_signal(||Vec::new());
    let move_page=use_signal(||0);

    use_load_book(book());
    use_css_injector();
    chapter_fetch_hook(html_vec);
    page_navigator(move_page, html_vec, visible_chunks);

    global_updater();

    rsx! {
        h1 {
            "{book}",
        }
        div {
            id: "book-renderer",
            style: "
                position: relative;
                width: 100vw;
                box-sizing: border-box;
                overflow-wrap: break-word;
                word-wrap: break-word;
                overflow-x: hidden;
                padding: 1rem;
            ",
            BookRenderer { visible_chunks }
        }
        BookButtons { move_page}
    }
 }