use dioxus::{logger::tracing, prelude::*};


use crate::{Route, domain::{self, book::{BookData, use_book}}, styles, ui::{ components::{TopBar, TopBarEntry, card::Cover}}};

#[component]
pub fn Book(book_id: String) -> Element {
    let book = use_book(book_id.clone());
    let b = book_id.clone();
    let nav = use_navigator();

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
            TopBar { entries: top_entries, show_extra: use_signal(||true), text_extra: Some("Edit".to_string()), on_extra: Callback::new(move |_| {nav.push(Route::BookEdit { book_id: b.clone() });}) }

            div {
                style: "
                    display: flex;
                    flex-direction: row;
                    gap: 20px;
                    flex: 1;
                    padding: 20px;
                    flex-wrap: wrap;
                ",
                Cover {
                    book_id: book_id.clone(),
                    width: "200px".to_string(),
                    max_width: "300px".to_string(),
                }

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

                div {
                    style: "
                        display: flex;
                        flex-direction: column;
                        gap: 5px;
                        flex: 0 0 200px;
                        max-height: 400px;
                    ",
                    h3 { "Chapters:" }
                    // key forces Dioxus to unmount + remount ChapterList
                    // whenever the current chapter changes, instead of
                    // trying to patch the existing DOM subtree.
                    ChapterList {
                        key: "{book().current_chapter}",
                        chapters: book().chapters.clone(),
                        current: book().current_chapter,
                        book,
                        book_id: book_id.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn ChapterList(
    chapters: Vec<String>,
    current: usize,
    book: Signal<BookData>,
    book_id: String,
) -> Element {
    // Only fetch what this component actually owns: chapter progress.
    let progress = domain::book::get_chapter_progress(book_id.clone());
    tracing::info!("loading chapter list");
    use_effect(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let (Some(current_elem), Some(list_elem)) = (
                    document.get_element_by_id("current-chapter"),
                    document.get_element_by_id("chapterlist"),
                ) {
                    let top = (current_elem.get_bounding_client_rect().top()
                        - list_elem.get_bounding_client_rect().top())
                        .floor();
                    list_elem.set_scroll_top(top as i32);
                } else {
                    tracing::error!("could not find chapter list elements");
                }
            }
        }
    });

    use_effect(move || {
        let current_id = format!("chapter-row-{current}");
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let (Some(current_elem), Some(list_elem)) = (
                    document.get_element_by_id(&current_id),
                    document.get_element_by_id("chapterlist"),
                ) {
                    let top = (current_elem.get_bounding_client_rect().top()
                        - list_elem.get_bounding_client_rect().top())
                        .floor();
                    list_elem.set_scroll_top(top as i32);
                }
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
                scrollbar-width: none;
                -ms-overflow-style: none;
            ",
            id: "chapterlist",
            class: "no-scrollbar",
            {
                chapters.iter().enumerate().map(|(idx, chapter)| {
                    let is_current = idx == current;
                    tracing::debug!("current chapter check for: {}, {}", idx, is_current);
                    let id = book_id.clone();
                    let row_id = format!("chapter-row-{idx}");
                    rsx! {
                        li {
                            key: "{idx}-{is_current}",
                            class: if is_current { "current-chapter" } else { "" },
                            id: "{row_id}",
                            button {
                                key: "{idx}-{is_current}",
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
                                        key: "{idx}-{is_current}",
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