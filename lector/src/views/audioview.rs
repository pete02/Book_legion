use std::collections::HashMap;

use dioxus::prelude::*;
use crate::components::audio::{AudioPlayer, ControlButtons, TimeBar, use_audio_chunk_loader, use_chunk_calculator, use_playback_tick};
use crate::components::{BookCover, use_book_parsing, use_load_book};
use crate::models::{ChunkProgress};


#[component]
pub fn AudioView( ) -> Element {
    let playing= use_signal(||false);
    let forward=use_signal(|| false);
    let backward=use_signal(|| false);

    let total_played = use_signal(|| 0.0);
    let chunkmap=use_signal(||None::<HashMap<String,ChunkProgress>>);
    let audio_url=use_signal(|| None::<String>);

    let book=use_signal(||"".to_string());
    



    use_load_book("mageling".to_string(), total_played);
    use_book_parsing(book);
    use_chunk_calculator(total_played, chunkmap);
    use_playback_tick(playing, total_played);
    use_audio_chunk_loader(total_played, audio_url, chunkmap);

    rsx! {
        div {
            class: "min-h-screen flex flex-col items-center justify-start",
            h1 { "Audio View" }
            AudioPlayer { playing, audio_url }           
            BookCover {name: book}
        
            div {
                class: "w-full flex flex-col items-center justify-start",
                TimeBar { total_played, audio_url }
                ControlButtons {playing, forward, backward}
            }   
         }
    }
}
