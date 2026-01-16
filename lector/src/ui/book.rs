use dioxus::{ prelude::*};

use crate::{Route, domain::{self, book::{BookData, load_book}}, styles, ui::components::{TopBar, TopBarEntry, card::Cover}};


#[component]
pub fn Book(book_id:String)->Element{
    let book=load_book(book_id);
        let top_entries = vec![
        TopBarEntry {
            name: "Library".into(),
            path: Route::Library {  }
        },
        TopBarEntry {
            name: "Series".into(),
            path: Route::Series { series_id: book().series_id }
        },
    ];

    return rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",            
            TopBar{ entries: top_entries }

            div {
                style: styles::BOOK_CONTAINER,

                Cover  {
                    cover_path: use_signal(||book.read().cover.clone()),
                    width: "200px".to_string(),
                    max_width: "300px".to_string(),
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
                    ChapterList { book }

                }
            }
        }
    }
}


#[component]
fn ChapterList(book: Signal<BookData>) -> Element {
    let chapters = book().chapters.clone();
    let current = book().current_chapter;

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

                    rsx! {
                        li {
                            button {
                                onclick: move |_| {
                                   domain::book::select_chapter(book, idx);
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
                                "{chapter}"
                            }
                        }
                    }
                })
            }
        }
    }
}
