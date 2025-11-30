use dioxus::html::geometry::euclid::num::Floor;
use dioxus::logger::tracing;
use dioxus::prelude::*;

use web_sys::{HtmlAudioElement};
use wasm_bindgen::JsCast;

use crate::assets::{PAUSE, PLAY};


use crate::models::GlobalState;

#[component]
pub fn AudioControls(current: Signal<f64>, playing: Signal<bool>) -> Element {
    let globalstate = use_context::<Signal<GlobalState>>();
    let total=use_signal(||0.0);
    let total_str=use_signal(||"".to_owned());
    let cur_str=use_signal(||"".to_owned());
    let precent= use_signal(||0.0);

    use_effect({
        let mut total=total.clone();
        let mut total_str=total_str.clone();

        move ||{
            match globalstate().book {
                None => total.with_mut(|f| *f=0.0),
                Some(book)=> {
                    let t= book.duration as i32;
                    tracing::debug!(t);
                    total.set(book.duration);
                    total_str.set(format!("{:02}:{:02}:{:02}", t/3600, (t/60).floor(), t%60));
                }
            }
        }
    });


    use_effect({
        let mut cur_str=cur_str.clone();
        let mut precent=precent.clone();
        move || {
            precent.set((current()/total())*100.0);
            let t=current() as i32;
            cur_str.set(format!("{:02}:{:02}:{:02}", t/3600, (t/60)%60, t%60));
        }
    });

    rsx! {
        div {
            class: "w-full flex flex-col items-center",  // center everything horizontally
            div {
                class: "w-[90%] bg-gray-300 h-3 rounded-full overflow-hidden", // progress bar 90% width
                div {
                    class: "h-full bg-blue-500 transition-all duration-300",
                    style: "width: {precent}%;",
                }
            }
            div {
                class: "w-[90%] mt-1", // same width as progress bar
                p {
                    class: "text-left text-sm", // left-aligned
                    "{cur_str()} / {total_str()}"
                }
            }
            

            div {
                class: "my-2", // vertical spacing
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