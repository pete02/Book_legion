use dioxus::prelude::*;

use crate::domain::book::{get_book_info, update_book};

#[component]
pub fn BookEdit(book_id: String) -> Element {
    let book = get_book_info(book_id.clone());

    // editable copy
    let mut draft = use_signal(|| book());

    // keep draft in sync when book loads
    use_effect(move || {
        draft.set(book());
    });

    rsx! {
        div {
            h1 { "Edit Book" }

            label { "Title" }
            input {
                value: "{draft().title}",
                oninput: move |evt| {
                    let mut b = draft();
                    b.title = evt.value();
                    draft.set(b);
                }
            }

            label { "Series ID" }
            input {
                value: "{draft().series_id}",
                oninput: move |evt| {
                    let mut b = draft();
                    b.series_id = evt.value();
                    draft.set(b);
                }
            }

            label { "Series Order" }
            input {
                value: "{draft().series_order}",
                oninput: move |evt| {
                    if let Ok(v) = evt.value().parse::<i32>() {
                        let mut b = draft();
                        b.series_order = v as usize;
                        draft.set(b);
                    }
                }
            }

            button {
                onclick: move |_| {
                    update_book(draft());
                },
                "Save"
            }
        }
    }
}