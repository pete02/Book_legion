use dioxus::prelude::*;

use web_sys::{HtmlAudioElement};
use wasm_bindgen::JsCast;

use crate::assets::{PAUSE, PLAY};


#[component]
pub fn AudioButtons(playing: Signal<bool>)->Element{

    rsx!{
        div {
            button {
            class: "w-12 h-12 flex items-center justify-center",
            onclick: move |_| {
                playpause(playing);
            },
              img {
                    class: "w-full h-full object-contain",
                    src: if *playing.read() { PAUSE } else { PLAY }
                }
            }
        }
    }
}


fn playpause(playing: Signal<bool>){
    let document = web_sys::window().unwrap().document().unwrap();
    if let Some(audio) = document.get_element_by_id("my_audio") {
        if *playing.read(){
            let audio: HtmlAudioElement = audio.dyn_into().unwrap();
            let _ = audio.pause();
        }else{
            let audio: HtmlAudioElement = audio.dyn_into().unwrap();
            let _ = audio.play();
        }
    }
}