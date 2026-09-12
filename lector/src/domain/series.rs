use dioxus::{logger::tracing, prelude::*};

use crate::infra::{self, series};
use crate::domain::cover::{CardData, create_cover_path};

async fn load_series(series_id: String, title: Signal<String>) -> Result<Vec<CardData>, Box<dyn std::error::Error>>{
    let mut books=Vec::new();
    let mut title=title.clone();

    let mut series=series::fetch_series(&series_id).await?;
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


#[allow(dead_code)]
pub fn delete_series(series_id:String){
    spawn(async move{
        match infra::series::delete_series(&series_id).await{
            Ok(_)=>{}
            Err(_)=>error!("Could not delete the series")
        }
    });
}

// domain/series.rs
pub fn get_book_count(series_id: Memo<String>) -> Resource<i32> {
    use_resource(move || {
        let series_id = series_id(); // reactive read — tracked, so this reruns whenever the memo changes
        async move {
            if series_id.is_empty() {
                return 0;
            }
            match infra::series::fetch_series(&series_id).await {
                Ok(c) => c.len() as i32,
                Err(e) => {
                    tracing::error!("Err in loading series book count: {}", e);
                    0
                }
            }
        }
    })
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