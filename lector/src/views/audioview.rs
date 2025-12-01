use dioxus::prelude::*;
use crate::components::audio::{AudioPlayer, ChunkCalculator, AudioControls};
use crate::components::LoadBook;
use crate::models::GlobalState;


#[component]
pub fn AudioView( ) -> Element {
    let playing= use_signal(||false);
    let total_played = use_signal(|| 0.0);

    let book=use_signal(||"".to_string());

    use_effect({
        let mut book=book.clone();
        let global=use_context::<Signal<GlobalState>>();
        move || {
            match global().book {
                None=>{},
                Some(b)=>{book.set(b.name);}
            }
        }
    });

    rsx! {
        div {
            class: "min-h-screen flex flex-col items-center justify-start",
            h1 { "Audio View" }
            AudioPlayer { playing, total_played }            

            {
                if book().len() > 0 {
                    rsx! {
                        img { 
                            class: "w-[90%] max-w-[400px] h-auto object-contain rounded-xl shadow-md",
                            src: "http://127.0.0.1:8000/cover/{book}" 
                        }
                    }
                } else {
                    rsx! { Fragment {} }
                }
            }
            div {
                class: "w-full flex flex-col items-center justify-start", // optional if needed
                AudioControls { current: total_played, playing }
            }

            LoadBook { book_name:"mageling", time:total_played }

            ChunkCalculator { time:total_played }
        }
    }
}
