use std::thread::current;

use dioxus::{logger::{self, tracing}, prelude::*};

use crate::{Route, domain::{self, text}, infra, ui::components::{TopBar, TopBarEntry}};

#[component]
pub fn Text(book_id: String) -> Element {
    let b_signal = use_signal(|| book_id.clone());
    let show_extra = use_signal(|| false);

    let chapter_html: Signal<Option<String>> = use_signal(|| None);
    let mut current_page: Signal<i32> = use_signal(|| -1);
    let mut column_width_px: Signal<Option<f64>> = use_signal(|| None);
    let mut total_pages: Signal<Option<i32>> = use_signal(|| None);
    let mut offset: Signal<Option<i64>> = use_signal(|| Some(0));
    let mut go_to_last_page: Signal<bool> = use_signal(|| false);
    let mut chapter_idx: Signal<usize> = use_signal(|| usize::MAX);
    let mut is_restoring: Signal<bool> = use_signal(|| true);

    let mut moved_page: Signal<bool> = use_signal(|| false);


    use_effect(move || {
        spawn(async move{
            tracing::debug!("load cursro");
            let bc=domain::cursor::load_bookcursor(b_signal()).await;
            chapter_idx.set(bc.cursor.chapter);
            offset.set(Some(bc.cursor.index as i64));
            tracing::info!("cursor: {:?}", bc.cursor);
        });
    });

    use_effect(move || {

        if chapter_idx() == usize::MAX{
            return;
        }

        let book_id = b_signal();
        spawn(async move {
            text::get_new_chapter(chapter_idx(), &book_id, chapter_html).await;
            
        });
    });
    
    use_effect(move || { 
        tracing::info!("start checking page offset");
        if moved_page() {
            return;
        }

        let Some(saved_offset) = offset() else {
            return;
        };
        tracing::info!("page offset required: {}", saved_offset);

        spawn(async move {
            // Let the browser finish the resize/reflow first.
            gloo_timers::future::TimeoutFuture::new(50).await;
            if moved_page() {
                return;
            }
            let Some(width) = text::measure_element_width("book-renderer") else {
                tracing::error!("no width could be measured");
                return;
            };
            column_width_px.set(Some(width));
            if let Some(page) = text::resolve_offset_to_page_with_retry(saved_offset, width).await {
                tracing::info!("resolved page: {}", page);
                is_restoring.set(false);
                current_page.set(page);
            }else{
                tracing::error!("could not resolve page")
            }
        });
    });



    use_effect(move || {
        if chapter_html().is_none() {
            return;
        }
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            if let Some(width) = text::measure_element_width("book-renderer") {
                column_width_px.set(Some(width));
            }
        });
    });

    use_effect(move || {
        if let Some(html) = chapter_html() {
            spawn(async move {
                let mut eval = document::eval(r#"
                    const html = await dioxus.recv();
                    const host = document.getElementById("book-content");

                    if (host) {
                        const root = host.shadowRoot ?? host.attachShadow({ mode: "open" });

                        root.innerHTML = `
                            <style>
                                img {
                                    display: block !important;
                                    margin-left: auto !important;
                                    margin-right: auto !important;
                                    max-width: 100% !important;
                                    max-height: 90dvh !important;
                                    object-fit: contain;
                                    break-inside: avoid;
                                    page-break-inside: avoid;
                                }
                            </style>
                            ${html}
                        `;
                    }
                "#);

                let _ = eval.send(html);
                total_pages.set(text::measure_total_pages("book-content"))
            });
        }
    });

    use_effect(move ||{
        if current_page()==-1 || *is_restoring.peek() || !moved_page(){ 
            return
        }

        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            if let Some(index) = text::measure_current_page_html_offset().await {
                text::save_cursor(current_page(), index, chapter_idx(), &b_signal()).await;
            }
        });
    });


    let top_entries = vec![
        TopBarEntry { name: "Library".into(), path: Route::Library {} },
        TopBarEntry { name: "Book".into(), path: Route::Book { book_id: book_id.clone() } },
    ];

    if chapter_html().is_none() {
        return rsx!(div { id: "book-renderer", "Loading reader…" });
    }

    let page = current_page();

    let column_style = match column_width_px() {
        Some(w) => format!("column-width: {w}px; column-gap: 0; column-fill: auto;"),
        None => String::new(),
    };
    let transform_style = format!("transform: translateX(-{}00%);", page);

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
                    div {
                        id: "book-content",
                        style: "height: 100%; {column_style} {transform_style}",
                    }
                }
                div {
                    id: "page-nav-overlay",
                    style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; display: flex;",
                    button {
                        style: "flex: 1 1 0; cursor: pointer; background: transparent;",
                        disabled: *current_page.read() <= 0 && chapter_idx() == 0,
                        onclick: move |_| {
                            moved_page.set(true);
                            let id = b_signal();
                            let should_load_prev_chapter = *current_page.read() <= 0;

                            if !should_load_prev_chapter {
                                *current_page.write() -= 1;
                            } else {
                                spawn(async move {
                                    chapter_idx.set(chapter_idx()-1);
                                    total_pages.set(None);
                                    go_to_last_page.set(true);
                                    gloo_timers::future::TimeoutFuture::new(0).await;
                                    text::get_new_chapter(chapter_idx(), &id, chapter_html).await;
                                    current_page.set(text::measure_total_pages("book-content").unwrap_or(0))
                                });
                            }
                        },
                    }
                    button {
                        style: "flex: 1 1 0; cursor: pointer; background: transparent;",
                        onclick: move |_| {
                            moved_page.set(true);
                            let id=b_signal();
                            if current_page() < text::measure_total_pages("book-content").unwrap_or(0) {
                                *current_page.write() += 1;
                            }else{
                                logger::tracing::debug!("change new  chapter");
                                current_page.set(0);
                                chapter_idx.set(chapter_idx() + 1);
                            }
                        },
                    }
                }
            }
        }
    }
}