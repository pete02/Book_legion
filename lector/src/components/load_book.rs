use dioxus::{logger::tracing, prelude::*};
use dioxus_signals::Signal;
use crate::models::{GlobalState, BookStatus};
use reqwasm::http::Request;


#[derive(Clone, PartialEq)]
enum LoadStatus {
    Loading,
    Success,
    Error(String),
}


pub fn load_book(book_name:String, time: Signal<f64>) {
    let global = use_context::<Signal<GlobalState>>();
    let status = use_signal(|| LoadStatus::Loading);
    let mut time=time.clone();

    use_effect(move || {
        let mut global = global.clone();
        let mut status = status.clone();
        let value = book_name.clone();
        spawn(async move {            
            tracing::info!("loading book: {}",&value);
            async fn get_book(book_name:String) -> Result<BookStatus, Box<dyn std::error::Error>> {
                let json: BookStatus = Request::get(&format!("http://127.0.0.1:8000/init?name={}&type=text",book_name))
                    .send()
                    .await?
                    .json()
                    .await?;
                Ok(json)
            }

            match get_book(value).await {
                Ok(book) => {
                    time.set(book.time.clone());
                    global.with_mut(|state| state.book = Some(book));
                    tracing::info!("Book loaded");
                    status.set(LoadStatus::Success);
                }
                Err(e) => {
                    tracing::error!("Error in book loading: {}",e);
                    status.set(LoadStatus::Error(format!("{}", e)));
                }
            }
        });
    });
}
