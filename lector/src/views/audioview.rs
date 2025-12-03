use std::collections::HashMap;

use dioxus::prelude::*;
use crate::components::audio::{AudioPlayer, ControlButtons, TimeBar, chunk_calculator, use_playback_tick};
use crate::components::{load_book,BookCover};
use crate::models::{ChunkProgress,  GlobalState};


#[component]
pub fn AudioView( ) -> Element {
    let playing= use_signal(||false);
    let forward=use_signal(|| false);
    let backward=use_signal(|| false);

    let total_played = use_signal(|| 0.0);
    let chunkmap=use_signal(||None::<HashMap<String,ChunkProgress>>);
    let audio_url=use_signal(|| None::<String>);

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


    use_playback_tick(playing, total_played);
    load_book("mageling".to_string(), total_played);
    chunk_calculator(total_played, chunkmap);

    rsx! {
        div {
            class: "min-h-screen flex flex-col items-center justify-start",
            h1 { "Audio View" }
            AudioPlayer { playing, total_played, chunkmap, audio_url }           
            BookCover {name: book}
        
            div {
                class: "w-full flex flex-col items-center justify-start",
                TimeBar { total_played, audio_url }
                ControlButtons {playing, forward, backward}
            }   
         }
    }
}
