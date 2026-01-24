use dioxus::{logger::tracing, prelude::*};

use crate::infra::series;
use crate::domain::cover::{CardData, create_cover_path};

async fn load_series(book_id: String, title: Signal<String>) -> Result<Vec<CardData>, Box<dyn std::error::Error>>{
    let mut books=Vec::new();
    let mut title=title.clone();

    let mut series=series::fetch_series(&book_id).await?;
    series.sort_by_key(|b|b.series_order);

    for entry in series{
        if title() == ""{
            if let Some(name)=entry.series_name{
                title.set(name);
            }
        }

        let b=CardData{
            name: entry.title,
            path: format!("/books/{}",entry.id),
            pic_path: create_cover_path(entry.id)
        };
        books.push(b);
    }

    return Ok(books);

}



pub fn use_series(book_id: String, title: Signal<String>) -> Signal<Vec<CardData>> {
    let mut books = use_signal(Vec::new);
    use_effect(move || {
        let  book_id=book_id.clone();
        spawn(async move {
            match load_series(book_id, title).await {
                Ok(data) => books.set(data),
                Err(e) => {
                    tracing::error!("Err in loading library: {}",e);
                    books.set(Vec::new());
                 }
            }
        });
    });

    books
}