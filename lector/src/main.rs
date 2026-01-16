use dioxus::prelude::*;
mod assets;
mod infra;
mod domain;
mod ui;
mod styles;
use crate::{domain::login, ui::{Library, LoginGuard, Series, Book}};

use assets::*;

fn main() {
    dioxus::launch(app);
}


#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Library{},

    #[route("/series/:series_id")]
    Series{ series_id: String },

    #[route("/books/:book_id")]
    Book{ book_id: String },

}

#[component]
fn app() -> Element {
    let load=login::restore_user_from_storage();

    let user = use_signal(|| load);

    use_context_provider(||user);

    rsx! {
        div {
            style: "min-height: 100vh; margin: 0; overflow: visible;",
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
            LoginGuard {Router::<Route> {}}
        }
    }
}





