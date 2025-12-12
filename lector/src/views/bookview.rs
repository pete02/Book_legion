use dioxus::{prelude::*};
use wasm_bindgen_futures::spawn_local;


use crate::{components::{BookCover, server_api, use_load_book}, models::GlobalState};

#[component]
pub fn BookView()->Element{
    let book=use_signal(||"fused".to_owned());
    let mut pressed=use_signal(||false);

    use_load_book(book());
    reset_book(pressed);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            h1 {
                style: "flex: 0 0 auto;",
                "{book}",
            },
            BookCover {name: book}

            a {
                href: "/ReadView",
                button { 
                    "Read"
                }
            }


            a {
                href: "/AudioView",
                button { 
                    "Listen"
                }
            }

            button { 
                onclick: move |_| {pressed.set(true); },
                "Reset"
             }
        }
    }
 }


 fn reset_book(mut pressed: Signal<bool>){
    let global=use_context::<Signal<GlobalState>>();

    use_effect(move || {
        if !pressed() {return;}

        match global().book{
            None=>{pressed.set(false);},
            Some(mut book)=>{
                book.chunk=1;
                book.chapter=book.initial_chapter;

                spawn_local(async move{
                    let _ = server_api::update_progress(book).await;
                    pressed.set(false);
                });
            }
    }
    });
 }