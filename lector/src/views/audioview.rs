use dioxus::prelude::*;
use crate::components::audio::{AudioPlayer, ControlButtons, TimeBar, audio_sourcing, use_chunk_calculator, use_playback_tick};
use crate::components::{BookCover, global_updater, load_name, use_book_parsing};
use crate::views::Route;



#[component]
pub fn AudioView( ) -> Element {
    let playing= use_signal(||false);
    let reload=use_signal(||true); // to load the book first time

    let time = use_signal(|| 0.0);

    let audio_url=use_signal(|| None::<String>);
    let book=use_signal(||"".to_string());
    
    use_book_parsing(book);
    use_chunk_calculator(time, reload);
    load_name(book);
    use_playback_tick(playing, time);

    audio_sourcing(audio_url, reload, time);
    global_updater();

    rsx! {
        div {
            class: "min-h-screen flex flex-col items-center justify-start",

            div {
                style: "display: flex; justify-content: flex-start; gap: 12px; align-items: center; padding: 8px 16px;",
                class: "bg-gray-200 dark:bg-gray-800 px-4 rounded-xl", 

                Link {
                    to: Route::LibraryView {  },
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Library"
                    }
                }
                Link{
                    to: Route::BookView {},
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Book"
                    }
                }

                Link{
                    to: Route::ReadView {},
                    button {
                        style: "padding: 8px 16px; border-radius: 8px; border: none; font-weight: bold;",
                        "Read"
                    }
                }
            },


            AudioPlayer { playing, audio_url }           
            BookCover {name: book}
        
            div {
                class: "w-full flex flex-col items-center justify-start",
                TimeBar { time, audio_url }
                ControlButtons {playing, time}
            }   
         }
    }
}
