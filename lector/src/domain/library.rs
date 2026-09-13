use dioxus::{logger::tracing, prelude::*};

use crate::infra::manifest::{self, ManifestEntry};
use crate::domain::cover::{CardData, create_cover_path};
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum LibraryMode {
    Normal,
    Alt(String), // the key itself, doubling as both flag and credential
}


/// Read/write access from anywhere below the provider.
pub fn use_library_mode() -> Signal<LibraryMode> {
    // Falls back to a fresh Normal-mode signal if no provider is above us,
    // rather than panicking. Still called synchronously in render — that part doesn't change.
    if let Some(signal) = try_consume_context::<Signal<LibraryMode>>() {
        signal
    } else {
        tracing::error!("could not consume library mode context");
        Signal::new(LibraryMode::Normal)
    }
}

pub fn read_library_mode() -> LibraryMode {
    use_library_mode().read().cloned()
}


pub fn get_library() -> Resource<Vec<ManifestEntry>> {
    use_resource(move || async move {
        manifest::fetch_manifest().await.unwrap_or_default()
    })
}

async fn load_library() -> Result<Vec<CardData>, Box<dyn std::error::Error>> {
    let mut books = Vec::new();
    let manifest = manifest::fetch_manifest().await?;

    for entry in manifest {
        books.push(CardData {
            name: entry.series_name,
            path: format!("/series/{}", entry.series_id),
            pic_path: create_cover_path(entry.first_book_id),
        });
    }
    Ok(books)
}

/// Sole owner of the fetch effect. Reactively refetches whenever mode changes —
/// no manual "update" call needed anywhere else.
pub fn use_library(mode: Signal<LibraryMode>) -> Signal<Vec<CardData>> {
    let mut books = use_signal(Vec::new);

    use_effect(move || {
        // Synchronous read here — this is what makes the effect track `mode`
        // and rerun automatically whenever it changes.
        let _ = mode();
        spawn(async move {
            match load_library().await {
                Ok(data) => books.set(data),
                Err(e) => {
                    tracing::error!("Err in loading library: {}", e);
                    books.set(Vec::new());
                }
            }
        });
    });

    books
}