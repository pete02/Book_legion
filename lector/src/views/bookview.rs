use dioxus::{prelude::*};
use wasm_bindgen_futures::spawn_local;


use crate::{components::{BookCover, server_api, use_load_book}, models::GlobalState, views::Route};

#[component]
pub fn BookView()->Element{
    let book=use_signal(||"fused".to_owned());
    let mut pressed=use_signal(||false);
    let loaded=use_signal(||false);

    use_load_book(book(), loaded);
    reset_book(pressed, loaded);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",

            // Top banner with buttons
            div {
                style: "display: flex; justify-content: flex-start; gap: 12px; align-items: center; padding: 8px 16px;",
                class: "bg-gray-200 dark:bg-gray-800 px-4", 

                Link {
                    to: Route::ReadView {  },
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Read"
                    }
                }
                
                Link{
                    to: Route::AudioView {},
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Listen"
                    }
                }

                button {
                    onclick: move |_| { pressed.set(true); },
                    style: "padding: 8px 16px; border-radius: 8px; border: none; background-color: #8B0000; color: white; font-weight: bold;",
                    "Reset"
                }
            }
            BookCover {
                name: book.clone(),
                width: "200px".to_string(),
                max_width: "300px".to_string(),
            }

            // Main content area
            div {
                style: "
                    flex: 1; 
                    display: flex; 
                    flex-direction: row; 
                    padding: 16px; 
                    gap: 16px; 
                    flex-wrap: wrap; 
                    justify-content: center;
                ",

                // Book Cover


                // Title + Description
                div {
                    style: "
                        flex: 1 1 300px;
                        display: flex;
                        flex-direction: column;
                        justify-content: flex-start;
                        gap: 8px;
                        min-width: 250px;
                        max-width: 600px;
                    ",

                    h1 {
                        style: "margin: 0; font-size: 1.8rem; font-weight: bold;",
                        "{book}"
                    }

                    p {
                        style: "font-size: 1rem;",
                        "This is a mock description for the book. It can be multiple lines and will wrap properly depending on screen size."
                    }
                }

                // Chapter list
                div {
                    style: "
                        flex: 1 1 200px; 
                        max-width: 250px; 
                        min-width: 150px; 
                        display: flex; 
                        flex-direction: column; 
                        gap: 4px; 
                        overflow-y: auto;
                    ",

                    h3 { "Chapters" }

                    ul {
                        style: "list-style: none; padding: 0; margin: 0;",
                        li { "Chapter 1: Introduction" }
                        li { "Chapter 2: Getting Started" }
                        li { "Chapter 3: Advanced Topics" }
                        li { "Chapter 4: Conclusion" }
                    }
                }
            }
        }
    }
}


 fn reset_book(mut pressed: Signal<bool>, mut loaded: Signal<bool>){
    let global=use_context::<Signal<GlobalState>>().clone();

    use_effect(move || {
        if !pressed() {return;}

        match global().book.clone() {
            None=>{pressed.set(false);},
            Some(mut book)=>{
                book.chunk=1;
                book.chapter=book.initial_chapter;

                spawn_local(async move{
                    let _ = server_api::update_progress(book).await;
                    pressed.set(false);
                    loaded.set(false);
                });
            }
    }
    });
 }