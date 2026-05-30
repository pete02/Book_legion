use dioxus::{ prelude::*};
use crate::{Route, assets, domain::{self, audio::{AudioData, skip_backward, skip_forward, use_audio}}, styles, ui::components::{TopBar, TopBarEntry, card::Cover}};



#[component]
pub fn Audio(book_id: String)->Element{
    let audio=use_audio(book_id.clone());
    let cover_path=domain::cover::create_cover_path(book_id.clone());
    let show_extra=use_signal(||false);

    let top_entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {  }
        },
        TopBarEntry {
            name: "Book".into(),
            path: Route::Book { book_id:book_id.clone() }
        },
    ];

    return rsx!{
        div {
        style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",            
            TopBar{ entries: top_entries, show_extra: show_extra }
            h1{ 
                style: styles::HEADER_STYLE,
                "{audio.name}"
            }
            div {
                style:  "display: flex; justify-content: center; align-items: center;",
                Cover {
                    cover_path: cover_path.clone(),
                }
            }
            TimeBar {audio: audio.clone()}
            ControlButtons {audio: audio.clone()}
            AudioPlayer { audio: audio.clone() }
        }
    }
}



#[component]
pub fn TimeBar(audio: AudioData) -> Element {
    let progress=audio.progress.clone();
    rsx! {
        div {
            class: "w-full flex flex-col items-center",
            div {
                class: "w-[90%] relative",
                div {
                    class: "bg-gray-300 h-3 rounded-full overflow-hidden w-full",
                    div {
                        class: "h-full bg-blue-500 transition-all duration-300 rounded-full",
                        style: "width: {(progress() * 100.0).round()}%;",
                    }
                }
                p {
                    class: "absolute left-0 -bottom-6 text-sm",
                    "{(progress() * 100.0).round()}%"
                }
            }
        }
    }
}

#[component]
pub fn ControlButtons(audio:AudioData )->Element{
    rsx! {
        div {
            style:  "display: flex; justify-content: center; align-items: center;",
            class: "my-2 flex items-center gap-4",
            style: "margin-top: 15px;",

            button {
                class: "
                    w-12 h-12 flex items-center justify-center
                    rounded-xl
                    bg-gray-200 dark:bg-gray-700
                    shadow 
                    transition active:scale-90 hover:bg-gray-300 dark:hover:bg-gray-600
                ",
                onclick: move |_| { skip_backward(audio) },
                img {
                    class: "w-6 h-6 object-contain transform -scale-x-100",
                    src: assets::FORWARD
                }
            }
            div {
                class: "relative w-14 h-14 overflow-visible",

                button {
                    class: "w-full h-full flex items-center justify-center transition active:scale-90",
                    onclick: move |_| { if !(*audio.error.read()) {
                            domain::audio::playpause(audio.play);
                        }
                    },
                    img {
                        class: "w-full h-full object-contain",
                        src: if *audio.error.read() { 
                            assets::ERROR
                        } else if *audio.play.read() { 
                            assets::PAUSE 
                        } else { 
                            assets::PLAY 
                        }
                    }
                }

                if *audio.loading.read() {
                    img {
                        class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 animate-spin pointer-events-none max-w-none",
                        style: "width: 130px; height: 130px;",
                        src: assets::LOADING
                    }
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
                onclick: move |_| {skip_forward(audio);},
                img {
                    class: "w-6 h-6 object-contain",
                    src: assets::FORWARD
                }
            }
        }
    }
}

#[component]
fn AudioPlayer(mut audio:AudioData) -> Element {
    return rsx! {
        div { id: "audio-player",
            audio {
                id: "my_audio",
                controls: false,
                style: "display:none",
                autoplay: "{audio.play}",
                src: "{audio.audio_url}",
                onplay: move |_| {
                    domain::wake::on_audio_play();
                    audio.play.set(true)
                },
                onended: move |_| { 
                    domain::wake::on_audio_pause();
                    domain::audio::switch_audio(audio.clone());
                },
                onpause: move |_|{
                    domain::wake::on_audio_pause();
                }
            }
        }
    }
}


