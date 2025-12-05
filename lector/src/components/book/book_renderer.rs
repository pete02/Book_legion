use dioxus::{ logger::tracing, prelude::*, web::WebEventExt};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use wasm_bindgen_futures::spawn_local;
use crate::models::{BookStatus, GlobalState};
use regex::Regex;

#[component]
pub fn BookRenderer(idle: Signal<bool>) -> Element {
    let html_vec: Signal<Vec<String>> = use_signal(Vec::new);
    let mut visible_chunks: Signal<Vec<String>> = use_signal(Vec::new);
    let mut container_ref: Signal<Option<web_sys::HtmlElement>> = use_signal(|| None);
    let start_index=use_signal(||0);

    let html_vec = html_vec.clone();
    let idle = idle.clone();

    use_effect(move || {
        if idle() {
            return;
        }

        let Some(book) = use_context::<Signal<GlobalState>>()().book else { return };
        let mut html_vec = html_vec.clone();

        spawn_local(async move {
            match fetch_page(book).await {
                Ok(chapter) => {
                    let vec = chapter
                        .split("\n")
                        .map(|s| s.to_string())
                        .filter(|s| !s.trim().is_empty())
                        .collect::<Vec<_>>();
                    tracing::debug!("vec set with {} el", vec.len());
                    html_vec.set(vec);
                }
                Err(e) => tracing::error!(?e, "Error fetching chapter"),
            }
        });
    });

    use_effect(move || {
        let Some(container) = container_ref() else { return; };
        let chunks = html_vec();
        if chunks.is_empty() { return; }
        let start_index=start_index.clone();
        let container_height = container.offset_height() as f64;
        let document = web_sys::window().unwrap().document().unwrap();

        let mut accumulated_height = 0.0;
        let mut fit_chunks = Vec::new();
        tracing::debug!("start: {}", start_index);
        for chunk in chunks.iter().skip(start_index()) {
            let temp = document.create_element("div").unwrap();
            temp.set_inner_html(chunk);
            temp.set_attribute("class", "chapter-chunk mb-2").unwrap();
            container.append_child(&temp).unwrap();
            let height = temp.unchecked_ref::<HtmlElement>().offset_height() as f64;
            container.remove_child(&temp).unwrap();

            accumulated_height += height;
            if accumulated_height > container_height {
                break;
            }
            fit_chunks.push(chunk.clone());
        }
        visible_chunks.set(fit_chunks);
        
    });

    rsx!(
        div {
            style: "height: 100%",
            onmounted: move |mounted| {
                let el = mounted.as_web_event().unchecked_into::<web_sys::HtmlElement>();
                container_ref.set(Some(el));
            },
            {
                visible_chunks.iter().map(|chunk| rsx!(
                div {
                    class: "chapter-chunk mb-2",
                    dangerous_inner_html: "{chunk}",
                    }
                ))
            }
        }
    )
}

async fn fetch_page(book: BookStatus) -> Result<String, Box<dyn std::error::Error>> {
    let url = "http://127.0.0.1:8000/book";
    let bytes = reqwasm::http::Request::post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&book)?)
        .send()
        .await?;
    let text = bytes.text().await?;
    Ok(strip_headers(&text))
}

fn strip_headers(xhtml: &str) -> String {
    let re = Regex::new(
        r"(?s)^<\?xml[^>]*>\s*<html[^>]*>.*?<body[^>]*>\s*|\s*</body>\s*</html>\s*$",
    )
    .unwrap();
    re.replace_all(xhtml, "").to_string()
}
