use std::collections::HashMap;

use dioxus::prelude::*;
use crate::components::audio::{AudioPlayer, ChunkCalculator, ControlButtons, TimeBar, use_playback_tick};
use crate::components::{LoadBook,BookCover};
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

    rsx! {
        
        div { 
            LoadBook { book_name:"mageling", time:total_played }
            ChunkCalculator { time:total_played, chunkmap }

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
}
