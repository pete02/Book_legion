use dioxus::{logger::tracing, prelude::*};

use crate::infra::manifest::{self, ManifestEntry};
use crate::domain::cover::{CardData, create_cover_path};

async fn load_library() -> Result<Vec<CardData>, Box<dyn std::error::Error>>{
    let mut books=Vec::new();

    let manifest=manifest::fetch_manifest().await?;

    for entry in manifest{
        let b=CardData{
            name: entry.series_name,
            path: format!("/series/{}",entry.series_id),
            pic_path: create_cover_path(entry.first_book_id)
        };
        books.push(b);
    }

    return Ok(books);

}



pub fn get_library() -> Resource<Vec<ManifestEntry>> {
    use_resource(move || async move {
        manifest::fetch_manifest().await.unwrap_or_default()
    })
}

pub fn use_library() -> Signal<Vec<CardData>> {
    let mut books = use_signal(Vec::new);
    use_effect(move || {
        spawn(async move {
            match load_library().await {
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