use dioxus::{prelude::*};

use crate::ui::components::Card;
use crate::{domain};
use crate::styles;

#[component]
pub fn Library() -> Element {
    let library=domain::library::use_library();
    return rsx! {
        div { style: styles::CONTAINER_STYLE,

            div { 
                style: styles::GRID_STYLE,
                {
                    library.iter().map(|entry| rsx!( Card{ entry: entry.clone() } ))
                }
            }
        }
    }
}
