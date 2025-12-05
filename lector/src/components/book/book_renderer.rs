use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen_futures::spawn_local;
use crate::models::{BookStatus, GlobalState};
use regex::Regex;


#[component]
pub fn BookRenderer(idle:Signal<bool>)->Element{
    let html_vec: Signal<Vec<String>>= use_signal(||Vec::new());
    let idle=idle.clone();
    use_effect(move ||{
        if idle() {return;}
        let mut html_vec=html_vec.clone();
        let Some(book)= use_context::<Signal<GlobalState>>()().book else {return;};
        spawn_local(async move {
            match fetch_page(book).await {
                Err(e)=>tracing::error!(e),
                Ok(chapter)=>{
                    if chapter.contains("\n"){
                        let mut vec=html_vec();
                        for chunk in chapter.split("\n"){
                            vec.push(chunk.to_owned());
                        }
                        html_vec.set(vec);
                    }
                },
            };
        });

    });

    rsx!(
         div {
            class: "readview-container p-4 text-gray-900 dark:text-gray-100",
            {
                html_vec.iter().map(|chunk|rsx!(
                div {
                    class: "chapter-chunk mb-2",
                    dangerous_inner_html: "{chunk}"
                }
            ))
            }
        }
    )
}

async fn fetch_page(book: BookStatus) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:8000/book",);
    let bytes = reqwasm::http::Request::post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&book)?)
        .send()
        .await?;
    match bytes.text().await {
        Ok(t)=>Ok(strip_headers(&t)),
        Err(_)=> Err("error in getting text".into())
    }
}

fn strip_headers(xhtml: &str) -> String {
    // Regex for start and end boilerplate
    let re = Regex::new(r"(?s)^<\?xml[^>]*>\s*<html[^>]*>.*?<body[^>]*>\s*|\s*</body>\s*</html>\s*$").unwrap();
    re.replace_all(xhtml, "").to_string()
}