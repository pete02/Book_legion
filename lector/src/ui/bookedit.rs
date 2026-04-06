use dioxus::prelude::*;
use crate::{
    Route, domain::{
        book::{ get_book_info},
        library::get_library
    }, infra, ui::components::{TopBar, TopBarEntry}
};

#[component]
pub fn BookEdit(book_id: String) -> Element {
    let book = get_book_info(book_id.clone());
    let series_list = get_library();

    let mut draft = use_signal(|| None::<BookInfo>);

    use_effect(move || {
        if let Some(b) = book() {
            if !b.id.is_empty() {
                draft.set(Some(b));
            }
        }
    });

    let top_entries = vec![
        TopBarEntry { name: "Library".into(), path: Route::Library {} },
        TopBarEntry {
            name: "Book".into(),
            path: Route::Book { book_id: book_id.clone() },
        },
    ];

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",

            TopBar {
                entries: top_entries,
                show_extra: use_signal(|| false),
                text_extra: None,
                on_extra: Callback::new(|_| {}),
            }

            if draft().is_none() {
                div {
                    style: "display: flex; align-items: center; justify-content: center; flex: 1;",
                    p { style: "color: #6b7280;", "Loading…" }
                    button {
                        style: "
                            padding: 10px 24px;
                            background: transparent;
                            color: #ef4444;
                            border: 1px solid #ef4444;
                            border-radius: 6px;
                            font-size: 1rem;
                            cursor: pointer;
                            margin-right: auto;
                        ",
                        onclick: {
                            let book_id=book_id.clone();
                            move |_| {
                            let id = book_id.clone();
                            let nav = navigator();
                            if web_sys::window()
                                .and_then(|w| w.confirm_with_message("Delete this book? This cannot be undone.").ok())
                                .unwrap_or(false)
                            {

                                spawn(async move {
                                    if let Err(e) = infra::book::delete_book(&id).await {
                                        error!("failed to delete book: {}", e);
                                    }
                                    nav.push(Route::Library {});
                                });
                            }
                        }
                        },
                        "Delete"
                    }
                    
                }
            } else {
                {
                    let d = draft().unwrap();
                    rsx! {
                        div {
                            style: "
                                max-width: 600px;
                                margin: 40px auto;
                                padding: 0 20px;
                                display: flex;
                                flex-direction: column;
                                gap: 20px;
                            ",

                            h1 {
                                style: "font-size: 1.6rem; font-weight: bold; margin: 0;",
                                "Edit — {d.title}"
                            }

                            Field {
                                label: "Title",
                                child: rsx! {
                                    input {
                                        style: INPUT_STYLE,
                                        value: "{d.title}",
                                        oninput: move |evt| update_draft(draft, |b| b.title = evt.value()),
                                    }
                                }
                            }

                            Field {
                                label: "Author ID",
                                child: rsx! {
                                    input {
                                        style: INPUT_STYLE,
                                        value: "{d.author_id}",
                                        oninput: move |evt| update_draft(draft, |b| b.author_id = evt.value()),
                                    }
                                }
                            }

                            Field {
                                label: "Series",
                                child: rsx! {
                                    select {
                                        style: INPUT_STYLE,
                                        value: "{d.series_id}",
                                        onchange: move |evt| update_draft(draft, |b| b.series_id = evt.value()),
                                        {
                                            series_list()
                                            .unwrap_or_default()
                                            .into_iter()
                                            .map(|s| {
                                                let selected = s.series_id == d.series_id;
                                                rsx! {
                                                    option {
                                                        value: "{s.series_id}",
                                                        selected: selected,
                                                        "{s.series_name}"
                                                    }
                                                }
                                            })
                                        }
                                    }
                                }
                            }

                            if !d.series_id.is_empty() {
                                Field {
                                    label: "Order in Series",
                                    child: rsx! {
                                        input {
                                            style: INPUT_STYLE,
                                            r#type: "number",
                                            min: "1",
                                            value: "{d.series_order}",
                                            oninput: move |evt| {
                                                if let Ok(v) = evt.value().parse::<usize>() {
                                                    update_draft(draft, |b| b.series_order = v);
                                                }
                                            },
                                        }
                                    }
                                }
                            }

                            div {
                                style: "display: flex; gap: 10px; justify-content: flex-end;",

                                // Delete — left side, always available once draft is Some
                                button {
                                    style: "
                                        padding: 10px 24px;
                                        background: transparent;
                                        color: #ef4444;
                                        border: 1px solid #ef4444;
                                        border-radius: 6px;
                                        font-size: 1rem;
                                        cursor: pointer;
                                        margin-right: auto;
                                    ",
                                    onclick: {
                                        let book_id=book_id.clone();
                                        move |_| {
                                        let id = book_id.clone();
                                        let nav = navigator();
                                        if web_sys::window()
                                            .and_then(|w| w.confirm_with_message("Delete this book? This cannot be undone.").ok())
                                            .unwrap_or(false)
                                        {

                                            spawn(async move {
                                                if let Err(e) = infra::book::delete_book(&id).await {
                                                    error!("failed to delete book: {}", e);
                                                }
                                                nav.push(Route::Library {});
                                            });
                                        }
                                    }
                                    },
                                    "Delete"
                                }

                                button {
                                    style: "
                                        padding: 10px 24px;
                                        background: transparent;
                                        color: #6b7280;
                                        border: 1px solid #d1d5db;
                                        border-radius: 6px;
                                        font-size: 1rem;
                                        cursor: pointer;
                                    ",
                                    onclick: {
                                        let b = book_id.clone();
                                        move |_| {
                                            navigator().push(Route::Book { book_id: b.clone() });
                                        }
                                    },
                                    "Cancel"
                                }

                                button {
                                    style: "
                                        padding: 10px 24px;
                                        background: #3b82f6;
                                        color: white;
                                        border: none;
                                        border-radius: 6px;
                                        font-size: 1rem;
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| {
                                        let nav = navigator();
                                        if let Some(b) = draft() {
                                            spawn(async move {
                                                if let Err(e) = infra::book::save_book(&b).await {
                                                    error!("failed to save book: {}", e);
                                                }
                                                nav.push(Route::Library {});
                                            });
                                        }else{
                                            info!("No draft");
                                        }
                                    },
                                    "Save"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────
use crate::infra::book::BookInfo;
/// Immutable-friendly draft updater
fn update_draft(mut draft: Signal<Option<BookInfo>>, f: impl FnOnce(&mut BookInfo)) {
    if let Some(mut b) = draft() {
        f(&mut b);
        draft.set(Some(b));
    }
}

const INPUT_STYLE: &str = "
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 1rem;
    width: 100%;
    box-sizing: border-box;
    background: white;
    color: #111827;
";

/// Simple label + input wrapper
#[component]
fn Field(label: String, child: Element) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 6px;",
            label {
                style: "font-size: 0.85rem; font-weight: 600; color: #374151;",
                "{label}"
            }
            {child}
        }
    }
}