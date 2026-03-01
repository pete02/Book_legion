use dioxus::{logger::tracing, prelude::*};


use crate::{Route, domain::{self, book::{BookData, use_book}}, styles, ui::components::{TopBar, TopBarEntry, card::Cover}};


#[component]
pub fn Book(book_id: String) -> Element {
    let book = use_book(book_id.clone());
    let cover_path = domain::cover::create_cover_path(book_id.clone());

    let top_entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {},
        },
        TopBarEntry {
            name: "Series".into(),
            path: Route::Series { series_id: book().series_id },
        },
        TopBarEntry {
            name: "Listen".into(),
            path: Route::Audio { book_id: book_id.clone() },
        },
        TopBarEntry {
            name: "Read".into(),
            path: Route::Text { book_id: book_id.clone() },
        },
    ];

    return rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",
            TopBar { entries: top_entries, show_delete: Signal::new(false) }

            div {
                style: "
                    display: flex;
                    flex-direction: row;
                    gap: 20px;
                    flex: 1;
                    padding: 20px;
                    flex-wrap: wrap;
                ",
                // Cover stays at top/left
                Cover {
                    cover_path: cover_path,
                    width: "200px".to_string(),
                    max_width: "300px".to_string(),
                }

                // Book text
                div {
                    style: "
                        flex: 1 1 300px;
                        min-width: 200px;
                        display: flex;
                        flex-direction: column;
                        gap: 10px;
                    ",
                    h1 {
                        style: "margin: 0; font-size: 1.8rem; font-weight: bold;",
                        "{book().title}"
                    }
                    h2 { "{book().author}" }
                    p {
                        style: "font-size: 1rem;",
                        "This is a mock description for the book. It can be multiple lines and will wrap properly depending on screen size."
                    }
                }

                // Chapter list scrollable
                div {
                    style: "
                        display: flex;
                        flex-direction: column;
                        gap: 5px;
                        flex: 0 0 200px;
                        max-height: 400px;
                    ",
                    h3 { "Chapters:" }
                    ChapterList { book, book_id: book_id }
                }
            }
        }
    }
}

#[component]
fn ChapterList(book: Signal<BookData>, book_id: String) -> Element {
    let chapters = book().chapters.clone();
    let current = book().current_chapter;
    let progress = domain::book::get_chapter_progress(book_id.clone());
    let mut found_current=use_signal(||false);

    use_effect(move || {
            // Get the element by ID
            if !found_current(){return;}
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(current_elem) = document.get_element_by_id("current-chapter") {
                        if let Some(w)=document.get_element_by_id("chapterlist"){
                            let top=(current_elem.get_bounding_client_rect().top()-w.get_bounding_client_rect().top()).floor();
                            w.set_scroll_top(top as i32);
                            tracing::info!("setting top to: {}. w top: {}",top, w.get_bounding_client_rect().top());
                        }else{
                            tracing::error!("could not find the element")
                        }
                    }
                }else{
                    tracing::error!("not found")
                }
            }
        });

    return rsx! {
        ul {
            style: "
                list-style: none;
                padding: 0;
                margin: 0;
                overflow-y: auto;
                scrollbar-width: none; /* Firefox */
                -ms-overflow-style: none; /* IE 10+ */
            ",
            id: "chapterlist",
            class: "no-scrollbar", // For Chrome/Safari
            {
                chapters.iter().enumerate().map(|(idx, chapter)| {
                    let is_current = idx == current;
                    let id = book_id.clone();
                    found_current.set(true);
                    rsx! {
                        li {
                            class: if is_current { "current-chapter" } else { "" },
                            id: if is_current { "current-chapter" } else { "" },
                            button {
                                onclick: move |_| {
                                   domain::book::select_chapter(book, progress, idx, id.clone());
                                },
                                style: format!(
                                    "{} border-left: 4px solid {}; font-weight: {};",
                                    styles::CHAPTER_BUTTON,
                                    if is_current { "#3b82f6" } else { "transparent" },
                                    if is_current { "bold" } else { "normal" },
                                ),
                                span { "{chapter}" }

                                if is_current {
                                    span {
                                        style: "font-size: 0.85em; color: #6b7280;",
                                        "    {(progress() * 100.0).round()}%"
                                    }
                                }
                            }
                        }
                    }
                })
            }
        }
    }
}