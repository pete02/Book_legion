use dioxus::prelude::*;

use crate::domain::{self, book::get_book_progress};


#[derive(Clone, PartialEq)]
pub struct AudioChunk{
    url:String,
    cursor: domain::cursor::Cursor
}

#[derive(Clone, PartialEq)]
pub struct AudioData{
    pub book_id:Signal<String>,
    pub name: Signal<String>,
    pub audio_url: Signal<String>,
    pub playing: Signal<bool>,
    pub audio_urls: Signal<Vec<AudioChunk>>,
    pub progress: Signal<f64>,
}
fn error_audio(book_id: String)->AudioData{
    AudioData { 
        book_id: use_signal(||book_id),
        name: use_signal(||"error in getting audio data".to_string()),
        audio_url: use_signal(||"".to_owned()),
        progress: use_signal(||0.0),
        playing: use_signal(||false),
        audio_urls: use_signal(||vec![]),
    }
}

pub async fn load_audio(book_id:String, mut audio: AudioData){
    let book=domain::book::load_book(book_id.clone()).await;
    audio.book_id.set(book_id.clone());
    audio.name.set(book.title);
    audio.audio_url.set("".to_string());
    audio.progress.set(domain::book::get_book_progress(book_id.clone()).await);
    audio.playing.set(false);
    audio.audio_urls.set(vec![]);
}

pub fn use_audio(book_id: String) -> AudioData {
    let book =error_audio(book_id.clone());
    let v=book.clone();
    use_effect(move ||{
        let book=book.clone();
        let book_id=book_id.clone();
        spawn(async move{
            load_audio(book_id,book).await;
        });
    });
    v
}


pub fn switch_audio(audio:AudioData){
    let mut audio_url: Signal<String>=audio.audio_url.clone();
    let mut audio_urls=audio.audio_urls.clone();
    let mut progress=audio.progress.clone();
    use_effect(move||{
        audio_url.set("".to_owned());
        let mut urls=audio_urls();
        if urls.len()>0{
            let url=urls.remove(0);
            audio_url.set(url.url);
            audio_urls.set(urls);

            let book_id=audio.book_id.clone();
            spawn(async move{
                let mut curs=domain::cursor::load_bookcursor(book_id()).await;
                curs.cursor.chapter=url.cursor.chapter;
                curs.cursor.chunk=url.cursor.chapter;

                let _= domain::cursor::save_bookcursor(curs).await;
                progress.set(get_book_progress(book_id()).await);
            });
        }
    });
}