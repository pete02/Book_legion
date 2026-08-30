use dioxus::{logger::tracing};
#[derive(Clone, PartialEq, Eq)]
pub struct CardData {
    pub name: String,
    pub path: String,
    pub pic_path: String,
}
#[cfg(not(feature = "mock"))]
pub fn create_cover_path(id:String)->String{
    return id;
}

#[cfg(feature = "mock")]
pub fn create_cover_path(id:String)->String{
    return crate::assets::MOCK_COVER.to_string();
}

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::infra::book;
static COVER_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn cover_cache() -> &'static Mutex<HashMap<String, String>> {
    COVER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn get_cached_cover(book_id: &str, width_px:&str) -> Result<String, Box<dyn std::error::Error>> {
    // Fast path: already cached.
    let width=&width_px.replace("px", "");
    if let Some(url) = cover_cache().lock().unwrap().get(book_id) {
        dioxus::logger::tracing::debug!("cover cache hit for {book_id}");
        return Ok(url.clone());
    }

    // Slow path: fetch (mock or real, whichever is compiled in), then cache.
    let url = book::fetch_cover(book_id, width).await?;
    cover_cache()
        .lock()
        .unwrap()
        .insert(book_id.to_string()+width, url.clone());
    Ok(url)
}