use dioxus::prelude::*;
use crate::{
    Route,
    domain,
    infra,
    ui::components::{TopBar, TopBarEntry},
};
#[component]
pub fn SeriesEdit(series_id: String) -> Element {
    let title = use_signal(|| "".to_owned());
    let mut delete_signal = use_signal(|| true);

    let books: Signal<Vec<domain::cover::CardData>> = domain::series::use_series(series_id.clone(), title);

    use_effect(move || {
        delete_signal.set(books().is_empty());
    });

    // Preset draft from title as soon as it arrives
    let mut draft = use_signal(|| "Loading…".to_owned());

    use_effect(move || {
        if title().len() > 0 {
            draft.set(title());
        }
    });

    let top_entries = vec![
        TopBarEntry { name: "Library".into(), path: Route::Library {} },
        TopBarEntry {
            name: "Series".into(),
            path: Route::Series { series_id: series_id.clone() },
        },
    ];

    let delete_style = format!(
        "padding: 10px 24px;
         background: transparent;
         color: #ef4444;
         border: 1px solid #ef4444;
         border-radius: 6px;
         font-size: 1rem;
         cursor: pointer;",
    );

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; font-family: sans-serif;",

            TopBar {
                entries: top_entries,
                show_extra: use_signal(|| false),
                text_extra: None,
                on_extra: Callback::new(|_| {}),
            }

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
                    "Edit — {draft}"
                }

                // Only show the edit form when series has books
                if !delete_signal() {
                    Field {
                        label: "Series Name",
                        child: rsx! {
                            input {
                                style: INPUT_STYLE,
                                value: "{draft}",
                                oninput: move |evt| draft.set(evt.value()),
                            }
                        }
                    }

                    div {
                        style: "display: flex; gap: 10px; justify-content: flex-end;",

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
                            onclick: {
                                let series_id=series_id.clone();
                                move |_| {
                                let name = draft();
                                let id = series_id.clone();
                                let nav = navigator();
                                spawn(async move {
                                    if let Err(e) = infra::series::update_series_name(&id, &name).await {
                                        error!("failed to save series: {}", e);
                                    }
                                    nav.push(Route::Library {});
                                });
                            }
                            },
                            "Save"
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
                                let sid = series_id.clone();
                                move |_| {
                                    navigator().push(Route::Series { series_id: sid.clone() });
                                }
                            },
                            "Cancel"
                        }
                    }
                } else {
                    // Series is empty — only deletion is allowed
                    p {
                        style: "color: #6b7280; font-size: 0.95rem;",
                        "This series has no books. You may delete it."
                    }

                    div {
                        style: "display: flex; gap: 10px;",
                        button {
                            style: "{delete_style}",
                            onclick: move |_| {
                                let id = series_id.clone();
                                let nav = navigator();
                                if web_sys::window()
                                    .and_then(|w| w.confirm_with_message("Delete this series? This cannot be undone.").ok())
                                    .unwrap_or(false)
                                {
                                    spawn(async move {
                                        if let Err(e) = infra::series::delete_series(&id).await {
                                            error!("failed to delete series: {}", e);
                                        }
                                        nav.push(Route::Library {});
                                    });
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
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