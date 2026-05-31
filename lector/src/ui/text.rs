use dioxus::{logger::tracing, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{Route, domain, renderer, ui::components::{TopBar, TopBarEntry}};
use crate::renderer::{BookDriver, Direction};

#[component]
pub fn Text(book_id: String) -> Element {
    let b_signal = use_signal(|| book_id.clone());
    let css_ready: Signal<bool> = use_signal(|| false);
    let show_extra = use_signal(|| false);
    let mut driver: Signal<Option<BookDriver>> = use_signal(|| None);

    use_effect(move || {
        tracing::debug!("here");
        domain::text::fetch_and_apply_book_css(b_signal(), css_ready);
    });

    // Once CSS is ready, init the driver
    use_effect(move || {
        if !css_ready() { return; }
        spawn(async move {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let el = document
                .get_element_by_id("book-renderer")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();

            if let Some(d) = BookDriver::new(b_signal(), el).await {
                driver.set(Some(d));
                // Load first page
                if let Some(ref mut d) = *driver.write() {
                    d.go(Direction::Forward).await;
                }
            }
        });
    });

    let top_entries = vec![
        TopBarEntry { name: "Library".into(), path: Route::Library {} },
        TopBarEntry { name: "Book".into(), path: Route::Book { book_id: book_id.clone() } },
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
                    // Left — go back
                    button {
                        style: "flex: 1 1 0; cursor: pointer; background: transparent;",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(ref mut d) = *driver.write() {
                                    d.go(Direction::Back).await;
                                    let slice=renderer::get_save_slice(&d.chapter_html, d.char_position);
                                    tracing::error!("Saving cursor text{:?}", domain::cursor::save_cursor_text(&d.book_id, &slice, d.chapter_idx).await);

                                }
                            });
                        },
                    }
                    // Right — go forward
                    button {
                        style: "flex: 1 1 0; cursor: pointer; background: transparent;",
                        onclick: move |_| {
                            spawn(async move {
                                if let Some(ref mut d)=*driver.write(){
                                    d.go(Direction::Forward).await;
                                }
                            });
                        },
                    }
                }
            }
        }
    }
}