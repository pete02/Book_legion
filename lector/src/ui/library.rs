use dioxus::logger::tracing;
use dioxus::{prelude::*};

use crate::domain::library;
use crate::ui::components::Card;
use crate::{domain::login};
use crate::styles;

#[component]
pub fn Library() -> Element {
    let mut mode: Signal<library::LibraryMode> =library::use_library_mode();
    let mut query = use_signal(String::new);

    let mut library = library::use_library();

    return rsx! {
        div { style: styles::CONTAINER_STYLE,

            input {
                r#type: "text",
                placeholder: "Search your library...",
                style: styles::SEARCH_STYLE,
                value: "{query}",
                oninput: move |e| {
                    let value = e.value();
                    tracing::debug!("Search query: {}, pin: {}", value, login::current_pin());
                    if value == login::current_pin() && login::current_pin() != "" {
                        tracing::debug!("Pin matched, mode: {:?}", mode());
                        mode.set(match mode() {
                            library::LibraryMode::Normal => library::LibraryMode::Alt(value.clone()),
                            library::LibraryMode::Alt(_) => library::LibraryMode::Normal,
                        });
                        tracing::debug!("mode after set: {:?}", mode()); // does this ever print Alt, then later Normal?

                        query.set(String::new());
                        return;
                    }

                    query.set(value);
                },
            }

            div { 
                style: styles::GRID_STYLE,
                {
                    library.iter().filter(|e| e.name.to_ascii_lowercase().contains(&query().to_ascii_lowercase())).map(|entry| rsx!( Card{ entry: entry.clone() } ))
                }
            }
        }
    }
}