use dioxus::prelude::*;
use crate::components::audio::{AudioPlayer, ChunkCalculator, AudioControls};
use crate::components::LoadBook;


#[component]
pub fn AudioView( ) -> Element {
    let playing= use_signal(||false);
    let total_played = use_signal(|| 0.0);
    rsx! {
        div {
            class: "min-h-screen flex flex-col items-center justify-start",
            h1 { "Audio View" }
            AudioPlayer { playing, total_played }            

            div {
                class: "w-full flex flex-col items-center justify-start", // optional if needed
                AudioControls { current: total_played, playing }
            }


            LoadBook { book_name:"mageling", time:total_played }

            ChunkCalculator { time:total_played }
        }
    }
}
