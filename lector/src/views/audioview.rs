use dioxus::prelude::*;
use crate::components::audio::{AudioButtons, AudioPlayer, ChunkCalculator};
use crate::components::LoadBook;

#[component]
pub fn AudioView( ) -> Element {
    let playing= use_signal(||false);
    let total_played = use_signal(|| 0.0);
    rsx! {
        div {
            class: "w-screen flex flex-col items-center justify-center",
            h1 { "Audio View" }
            AudioPlayer { playing, total_played }
            AudioButtons {  playing}
            p { "Total played time: {total_played}" }

            LoadBook { book_name:"mageling", time:total_played }
            ChunkCalculator { time:total_played }
        }
    }
}
