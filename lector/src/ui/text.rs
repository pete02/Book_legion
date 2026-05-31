use dioxus::{logger::tracing, prelude::*};
use futures_util::StreamExt;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{Route, domain, ui::components::{TopBar, TopBarEntry}};
use crate::renderer::{BookDriver, Direction};

pub enum DriverMsg {
    Init,
    Go(Direction),
}

#[component]
pub fn Text(book_id: String) -> Element {
    let b_signal = use_signal(|| book_id.clone());
    let css_ready: Signal<bool> = use_signal(|| false);
    let show_extra = use_signal(|| false);

    use_effect(move || {
        tracing::debug!("here");
        domain::text::fetch_and_apply_book_css(b_signal(), css_ready);
    });

    let driver_coroutine = use_coroutine(|mut rx: UnboundedReceiver<DriverMsg>| {
        let book_id = book_id.clone();
        async move {
            let mut driver: Option<BookDriver> = None;

            while let Some(msg) = rx.next().await {
                match msg {
                    DriverMsg::Init => {
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        let el = document
                            .get_element_by_id("book-renderer")
                            .unwrap()
                            .dyn_into::<HtmlElement>()
                            .unwrap();

                        if let Some(mut d) = BookDriver::new(book_id.clone(), el).await {
                            d.go(Direction::Forward).await;
                            driver = Some(d);
                        }
                    }
                    DriverMsg::Go(dir) => {
                        if let Some(ref mut d) = driver {
                            d.go(dir).await;
                        }
                    }
                }
            }
        }
    });

    // Once CSS is ready, send Init — runs once when css_ready flips to true
    use_effect(move || {
        if !css_ready() { return; }
        driver_coroutine.send(DriverMsg::Init);
    });

    let top_entries = vec![
        TopBarEntry { name: "Library".into(), path: Route::Library {} },
        TopBarEntry { name: "Book".into(), path: Route::Book { book_id: b_signal() } },
    ];

    if !css_ready() {
        return rsx!(div { id: "book-renderer", "Loading reader…" });
    }

    rsx! {
        div {
            style: "display: flex; flex-direction: column; flex: 1 1 auto; min-height: 0; overflow: hidden;",
            TopBar { entries: top_entries, show_extra: show_extra }
            div {
                style: "position: relative; display: flex; flex-direction: column; overflow: hidden; flex: 1 1 auto; min-height: 95dvh;",
                id: "text-container",
                div {
                    id: "book-renderer",
                    style: "height: 90dvh; overflow: hidden; position: relative; width: 90%; margin-left: 5%; margin-bottom: 0%; padding-bottom: 10px; text-align: left !important;",
                }
                div {
                    style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; display: flex;",
                    button {
                        style: "flex: 1 1 0; cursor: pointer; background: transparent;",
                        onclick: move |_| {
                            driver_coroutine.send(DriverMsg::Go(Direction::Back));
                        },
                    }
                    button {
                        style: "flex: 1 1 0; cursor: pointer; background: transparent;",
                        onclick: move |_| {
                            driver_coroutine.send(DriverMsg::Go(Direction::Forward));
                        },
                    }
                }
            }
        }
    }
}