use dioxus::{logger::tracing, prelude::*};
#[derive(Clone, PartialEq, Eq)]
pub struct CardData {
    pub name: String,
    pub path: String,
    pub pic_path: String,
}

pub fn create_cover_path(id:String)->String{
    let s=format!("/api/v1/books/{}/cover",id);
    tracing::debug!("returning: {}",s);

    return s;
}

pub fn use_cover(path: String) -> Signal<Option<String>> {
    let mut cover_url = use_signal(|| None);
    tracing::debug!("got urL :{:?}", cover_url.read());
    use_effect(move || {
        let url = path.clone();
        if url.is_empty() {
            cover_url.set(None);
            return;
        }

        spawn(async move {
            match crate::infra::covers::fetch_cover(&url).await {
                Ok(url) => cover_url.set(Some(url)),
                Err(_) => cover_url.set(None),
            }
        });
    });

    cover_url
}
