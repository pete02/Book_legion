use dioxus::{  logger::tracing, prelude::*, web::WebEventExt};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, window};
use wasm_bindgen_futures::spawn_local;
use crate::models::{BookStatus, GlobalState};
use regex::Regex;

#[component]
pub fn BookRenderer(idle: Signal<bool>, css_idle: Signal<bool>) -> Element {
    let html_vec: Signal<Vec<String>> = use_signal(Vec::new);
    let mut visible_chunks: Signal<Vec<String>> = use_signal(Vec::new);
    let mut container_ref: Signal<Option<web_sys::HtmlElement>> = use_signal(|| None);
    let mut start_index=use_signal(||0);


    let mut flip=use_signal(||false);
    let mut change=use_signal(||false);
    let mut wait=use_signal(||false);

    let html_vec = html_vec.clone();
    let idle = idle.clone();

    chapter_fetch_hook(idle, html_vec);
    calculate_chunks(html_vec, visible_chunks, start_index, css_idle, flip, change);

    rsx!(
        div {
            id: "book-renderer",
            style: "
                position: relative;
                width: 100vw;
                box-sizing: border-box;
                overflow-wrap: break-word;
                word-wrap: break-word;
                overflow-x: hidden;
                padding: 1rem;
            ",
            onmounted: move |mounted| {
                let el = mounted.as_web_event().unchecked_into::<web_sys::HtmlElement>();
                container_ref.set(Some(el));
            },
            {
                visible_chunks.iter().map(|chunk| rsx!(
                    div {
                        class: "chapter-chunk mb-2",
                        style: "
                            width: 100%;
                            box-sizing: border-box;
                            word-break: break-word;
                        ",
                        dangerous_inner_html: "{chunk}",
                    }
                ))
            }

            // Overlay buttons
            div {
                style: "
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    display: flex;
                ",

                // Left button
                div {
                    style: "
                        width: 50%;
                        height: 100%;
                        cursor: pointer;
                        background: transparent;
                    ",
                    onclick: move |_| {
                        if start_index-visible_chunks().len() >0{
                            change.set(true);
                            start_index.set(start_index-visible_chunks.len());
                            flip.set(true);
                            visible_chunks.set(vec![]);
                            change.set(false);
                        }
                    }
                }

                // Right button
                div {
                    style: "
                        width: 50%;
                        height: 100%;
                        cursor: pointer;
                        background: transparent;
                    ",
                    onclick: move |_| {
                        if !(start_index()==html_vec().len()){
                            tracing::debug!("clicl");
                            visible_chunks.set(vec![]);
                        }
                    }
                }
            }
        }
    )
}




fn chapter_fetch_hook(idle: Signal<bool>, html_vec: Signal<Vec<String>>){
    use_effect(move || {
        if idle() {return;}

        let Some(book) = use_context::<Signal<GlobalState>>()().book else { return };
        let mut html_vec = html_vec.clone();

        spawn_local(async move {
            match fetch_chapter(book).await {
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
}

fn calculate_chunks(
    html_vec: Signal<Vec<String>>,
    mut visible_chunks: Signal<Vec<String>>,
    start_index:Signal<usize>,
    css_idle: Signal<bool>,
    mut flip:Signal<bool>,
    change:Signal<bool>,
){

    let mut start_index=start_index.clone();

    use_effect(move || {
        if change() {return;}

        let a=visible_chunks().len();
        tracing::debug!("triggered, start: {}", start_index());
        if css_idle() {return;}

        let Some(nav_height)=get_element_height("book-container") else {return;};
        let Some(book_height)=get_element_height("book-renderer") else {return;};
        tracing::debug!("book_height: {}", book_height);

        if book_height >= nav_height-100.0 {
            tracing::debug!("too big");
            if flip(){
                start_index.set(start_index+visible_chunks().len());
                flip.set(false);
            }
            tracing::debug!("end index: {}", start_index());
            return;
        }

        if html_vec().len() ==0 {return;}

        if start_index() == 0 && flip() {
            if flip(){
                start_index.set(start_index+visible_chunks().len());
                flip.set(false);
            }
            tracing::debug!("end index: {}", start_index());
            return;
        }



        tracing::debug!("index: {} of {}", start_index(),html_vec().len());
        if start_index() == html_vec().len() {
            if flip(){
                start_index.set(start_index+visible_chunks().len());
                flip.set(false);
            }
            tracing::debug!("end index: {}", start_index());
            return;
        }
        

        let mut v=visible_chunks();
        
        if !flip(){
            let string=&html_vec()[start_index()];
            v.push(string.to_owned());
            visible_chunks.set(v);
            start_index.set(start_index()+1);
        }else{
            tracing::debug!("smaller");
            let string=&html_vec()[start_index()-1];
            v.insert(0, string.to_owned());
            visible_chunks.set(v);
            start_index.set(start_index() - 1);    
        }
    });
    
}

async fn fetch_chapter(book: BookStatus) -> Result<String, Box<dyn std::error::Error>> {
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



fn get_element_height(id: &str) -> Option<f64> {
    let document = window()?.document()?;
    let element = document.get_element_by_id(id)?;
    let html_element: HtmlElement = element.dyn_into().ok()?;
    Some(html_element.offset_height() as f64)
}