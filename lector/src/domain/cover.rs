use dioxus::{logger::tracing};
#[derive(Clone, PartialEq, Eq)]
pub struct CardData {
    pub name: String,
    pub path: String,
    pub pic_path: String,
}
#[cfg(not(feature = "mock"))]
pub fn create_cover_path(id:String)->String{
    let s=format!("/api/v1/books/{}/cover",id);
    tracing::debug!("returning: {}",s);
    return s;
}

#[cfg(feature = "mock")]
pub fn create_cover_path(id:String)->String{
    return crate::assets::MOCK_COVER.to_string();
}
