use dioxus::html::geometry::euclid::num::Floor;
use dioxus::logger::tracing;
use dioxus::prelude::*;

use web_sys::{HtmlAudioElement};
use wasm_bindgen::JsCast;

use crate::assets::{FORWARD, PAUSE, PLAY};


use crate::models::GlobalState;

#[component]
pub fn TimeBar(total_played: Signal<f64>, audio_url: Signal<Option<String>>) -> Element {
    let total=use_signal(||0.0);
    let total_str=use_signal(||"".to_owned());
    let cur_str=use_signal(||"".to_owned());
    let precent= use_signal(||0.0);

    create_total_time(total, total_str);

    create_current_time(total_played, cur_str, precent, total);


    rsx! {
    div {
        class: "w-full flex flex-col items-center",
        div {
            class: "w-[90%] relative",
            div {
                class: "bg-gray-300 h-3 rounded-full overflow-hidden w-full",
                div {
                    class: "h-full bg-blue-500 transition-all duration-300 rounded-full",
                    style: "width: {precent}%;",
                }
            }
            p {
                class: "absolute left-0 -bottom-6 text-sm",
                "{cur_str()} / {total_str()}"
            }
        }

    
        }
    }

}


#[component]
pub fn ControlButtons(playing: Signal<bool>, forward:Signal<bool>, backward: Signal<bool>)->Element{
    let mut forward=forward.clone();
    let mut backward = backward.clone();

    rsx! {
        div {
            class: "my-2 flex items-center gap-4",
            button {
                class: "
                    w-12 h-12 flex items-center justify-center
                    rounded-xl
                    bg-gray-200 dark:bg-gray-700
                    shadow 
                    transition active:scale-90 hover:bg-gray-300 dark:hover:bg-gray-600
                ",
                onclick: move |_| { backward.set(true); },
                img {
                    class: "w-6 h-6 object-contain transform -scale-x-100",
                    src: FORWARD
                }
            }

            button {
                class: "w-14 h-14 flex items-center justify-center",
                onclick: move |_| { playpause(playing); },
                img {
                    class: "w-full h-full object-contain",
                    src: if *playing.read() { PAUSE } else { PLAY }
                }
            }
            button {
                class: "
                    w-12 h-12 flex items-center justify-center
                    rounded-xl
                    bg-gray-200 dark:bg-gray-700
                    shadow 
                    transition active:scale-90 hover:bg-gray-300 dark:hover:bg-gray-600
                ",
                onclick: move |_| { forward.set(true); },
                img {
                    class: "w-6 h-6 object-contain",
                    src: FORWARD
                }
            }
        }
    }
    
}




fn create_current_time(current: Signal<f64>, cur_str: Signal<String>, precent:Signal<f64>, total:Signal<f64>){

    use_effect({
        let mut cur_str=cur_str.clone();
        let mut precent=precent.clone();
        move || {
            precent.set((current()/total())*100.0);
            let t=current() as i32;
            cur_str.set(format!("{:02}:{:02}:{:02}", t/3600, (t/60)%60, t%60));
        }
    });

}

fn create_total_time(total:Signal<f64>, total_str:Signal<String>){
    use_effect({
        let mut total=total.clone();
        let mut total_str=total_str.clone();
        let globalstate = use_context::<Signal<GlobalState>>();

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