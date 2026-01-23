use dioxus::{ logger::tracing, prelude::*};

use crate::{Route, domain::{self, book::{BookData, use_book}}, styles, ui::components::{TopBar, TopBarEntry, card::Cover}};


#[component]
pub fn Book(book_id:String)->Element{
    let book=use_book(book_id.clone());
    let cover_path=domain::cover::create_cover_path(book_id.clone());
    let top_entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {  }
        },
        TopBarEntry {
            name: "Series".into(),
            path: Route::Series { series_id: book().series_id }
        },
        TopBarEntry {
            name: "Listen".into(),
            path: Route::Audio { book_id:book_id.clone() }
        },
        TopBarEntry {
            name: "Read".into(),
            path: Route::Text { book_id:book_id.clone() }
        },
    ];
    
    return rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",            
            TopBar{ entries: top_entries }

            div {
                style: styles::BOOK_CONTAINER,
                {
                    rsx!{
                        Cover  {
                            cover_path: cover_path,
                            width: "200px".to_string(),
                            max_width: "300px".to_string(),
                        }
                    }
                }

                div {
                    style: styles::BOOK_TEXT,

                    h1 {
                        style: "margin: 0; font-size: 1.8rem; font-weight: bold;",
                        "{book().title}"
                    }
                    h2 {
                        "{book().author}"
                      }

                    p {
                        style: "font-size: 1rem;",
                        "This is a mock description for the book. It can be multiple lines and will wrap properly depending on screen size."
                    }
                }

                div {
                    style: styles::CHAPTERLIST_CONTAINER,
                    ChapterList { book, book_id:book_id }

                }
            }
        }
    }
}


#[component]
fn ChapterList(book: Signal<BookData>, book_id: String) -> Element {
    let chapters = book().chapters.clone();
    let current = book().current_chapter;
    let progress=domain::book::get_chapter_progress(book_id.clone());

    return rsx! {
        h3 { "Chapters:" }

        ul {
            style: "
                list-style: none;
                padding: 0;
                margin: 0;
                overflow-x: hidden;
            ",

            {
                chapters.iter().enumerate().map(|(idx, chapter)| {
                    let is_current = idx == current;
                    let id=book_id.clone();
                    rsx! {
                        li {
                            button {
                                onclick: move |_| {
                                   domain::book::select_chapter(book, idx, id.clone());
                                },
                                style: format!(
                                "
                                    {}
                                    border-left: 4px solid {};
                                    font-weight: {};
                                    ",
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
