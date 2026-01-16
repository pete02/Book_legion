use dioxus::prelude::*;
#[derive(Clone, PartialEq, Eq)]
pub struct CardData {
    pub name: String,
    pub path: String,
    pub pic_path: String,
}



pub fn use_cover(path: Signal<String>) -> Signal<Option<String>> {
    let mut cover_url = use_signal(|| None);

    use_effect(move || {
        let url = path();
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
