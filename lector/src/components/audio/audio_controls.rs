use std::collections::HashMap;

use dioxus::logger::tracing;
use dioxus::prelude::*;

use web_sys::{HtmlAudioElement};
use wasm_bindgen::JsCast;

use crate::assets::{FORWARD, PAUSE, PLAY};


use crate::models::GlobalState;

#[component]
pub fn TimeBar(time: Signal<f64>, audio_url: Signal<Option<String>>) -> Element {
    let total=use_signal(||0.0);
    let total_str=use_signal(||"".to_owned());
    let cur_str=use_signal(||"".to_owned());
    let precent= use_signal(||0.0);

    create_total_time(total, total_str);
    create_current_time(time, cur_str);
    create_precentage(precent);

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
                "{cur_str()}"
            }
        }

    
        }
    }

}


#[component]
pub fn ControlButtons(playing: Signal<bool>, reload:Signal<i32> )->Element{
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
                onclick: move |_| { reload.set(-1); },
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
                onclick: move |_| { reload.set(1);  },
                img {
                    class: "w-6 h-6 object-contain",
                    src: FORWARD
                }
            }
        }
    }
    
}

fn create_current_time(current: Signal<f64>, cur_str: Signal<String>){

    use_effect({
        let mut cur_str=cur_str.clone();        move || {
            let t=current() as i32;
            cur_str.set(format_time(t));
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
                    total.set(book.duration);
                    total_str.set(format_time(t));
                }
            }
        }
    });
}


fn create_precentage(mut precent:Signal<f64>){
    let global=use_context::<Signal<GlobalState>>();
    use_effect(move||{
        let Some(book)=global().book.clone() else {return;};
        let cur=(calculate_max_chunks(book.initial_chapter, book.chapter-1, &book.chapter_to_chunk) +book.chunk)as f64;
        let max=calculate_max_chunks(book.initial_chapter, book.max_chapter, &book.chapter_to_chunk) as f64;
        precent.set((cur/max)*100.0);
    });
}


fn format_time(seconds: i32) -> String {
    format!("{:02}:{:02}:{:02}", seconds/3600, (seconds/60)%60, seconds%60)
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


fn calculate_max_chunks(start:u32, stop:u32, chapter_to_chunk:&HashMap<u32,u32>)->u32{
    let mut chuhks=0;
    for i in start..=stop{
        let Some(num)=chapter_to_chunk.get(&i) else{return chuhks;};
        chuhks+=num;
    }
    chuhks
}