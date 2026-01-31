use dioxus::prelude::*;
mod assets;
mod infra;
mod domain;
mod ui;
mod styles;
use crate::{domain::login, ui::{Library, LoginGuard, Series, Book, Audio, Text}};


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

    #[route("/books/:book_id/audio")]
    Audio{book_id: String},

    #[route("/books/:book_id/text")]
    Text{book_id: String},

    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },

} 
#[component]
pub(crate) fn PageNotFound(route: Vec<String>) -> Element {

        let nav = navigator();

        nav.replace(Route::Library {});

        rsx! { p {} }
}

#[component]
fn app() -> Element {
    let load=login::restore_user_from_storage();

    let user = use_signal(|| load);
    console_error_panic_hook::set_once();

    use_context_provider(||user);

    return rsx! {
        
        div {
            style: "min-height: 100vh; margin: 0; overflow: visible;",
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
            LoginGuard {Router::<Route> {}}

        }
    }
}






