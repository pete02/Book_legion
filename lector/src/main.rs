use dioxus::prelude::*;

mod views;
mod assets;
mod models;

mod components;
use assets::*;


use views::Route;

use crate::models::{ GlobalState};




fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let global = use_signal(|| GlobalState::new());

    use_context_provider(||global);
    rsx! {
        div { 
            style: "height: 100vh; margin: 0",
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
            Router::<Route> {}
        }
    }
}





