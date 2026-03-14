use dioxus::prelude::*;
use crate::{Route, styles};
use wasm_bindgen::JsCast;



#[derive(Clone, Debug, PartialEq)]
pub struct TopBarEntry {
    pub name: String,
    pub path: Route,
}
#[component]
pub fn TopBar(
    entries: Vec<TopBarEntry>,
    show_extra: Signal<bool>,
    text_extra: Option<String>,
    on_extra: Option<Callback<()>>,
) -> Element {
    let mut hamburger = use_signal(|| false);
    let mut open = use_signal(|| false);

    use_effect(move || {
        let mut check = move || {
            let Some(window) = web_sys::window() else { return };
            let Some(doc) = window.document() else { return };
            let Some(probe) = doc.get_element_by_id("topbar-probe") else { return };
            let Some(root)  = doc.get_element_by_id("topbar-root")  else { return };
            hamburger.set(probe.scroll_width() > root.client_width());
        };

        check(); // measure on first mount

        // Re-check whenever the window is resized
        let cb = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || check());
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
    });

    rsx! {
        div {
            id: "topbar-root",
            style: "{styles::TOPBAR} position: relative;",

            // ── Hidden probe ─────────────────────────────────────────────
            // Always in the DOM so we can measure the natural width of all
            // items without being constrained by the container.
            div {
                id: "topbar-probe",
                style: "position: absolute; visibility: hidden; pointer-events: none;
                        display: flex; gap: 12px; white-space: nowrap;",
                { nav_items(&entries) }
                { extra_button(show_extra, &text_extra, &on_extra) }
            }

            // ── Normal layout ─────────────────────────────────────────────
            if !hamburger() {
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    { nav_items(&entries) }
                    { extra_button(show_extra, &text_extra, &on_extra) }
                }
            }

            // ── Hamburger layout ──────────────────────────────────────────
            if hamburger() {
                div {
                    style: "display: flex; align-items: center; justify-content: space-between; width: 100%;",

                    button {
                        class: "bg-blue-600 hover:bg-blue-700 text-white font-semibold
                                py-2 px-4 rounded-lg transition-colors duration-150",
                        onclick: move |_| open.toggle(),
                        if open() { "✕" } else { "☰" }
                    }

                    if open() {
                        div {
                            style: "position: absolute; top: 100%; left: 0; right: 0; z-index: 50;
                                    background: #1d273eff; box-shadow: 0 4px 12px rgba(0,0,0,0.1);
                                    display: flex; flex-direction: column; gap: 8px; padding: 12px;",
                            { nav_items(&entries) }
                            { extra_button(show_extra, &text_extra, &on_extra) }
                        }
                    }
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn nav_items(entries: &[TopBarEntry]) -> Element {
    rsx! {
        { entries.iter().map(|entry| {
            let path = entry.path.clone();
            let name = entry.name.clone();
            rsx! {
                Link {
                    to: path,
                    button {
                        class: "bg-blue-600 hover:bg-blue-700 active:bg-blue-800
                                text-white font-semibold py-2 px-4
                                rounded-lg transition-colors duration-150",
                        "{name}"
                    }
                }
            }
        })}
    }
}

fn extra_button(
    show_extra: Signal<bool>,
    text_extra: &Option<String>,
    on_extra: &Option<Callback<()>>,
) -> Element {
    if show_extra() {
        if let Some(on_extra) = on_extra.clone() && let Some(text) = text_extra {
            return rsx! {
                button {
                    class: "bg-red-600 hover:bg-red-700 active:bg-red-800
                            text-white font-semibold py-2 px-4
                            rounded-lg transition-colors duration-150",
                    onclick: move |_| on_extra.call(()),
                    "{text}"
                }
            };
        }
    }
    rsx! {}
}