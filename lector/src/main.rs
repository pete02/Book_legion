use dioxus::prelude::*;
mod assets;
mod domain;
mod infra;
mod styles;
mod ui;

use crate::{
    domain::login,
    ui::{Audio, Book, BookEdit, Library, LoginGuard, Series, SeriesEdit, Text},
};

use assets::*;

fn main() {
    dioxus::launch(app);
}

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Library {},

    #[route("/series/:series_id")]
    Series { series_id: String },

    #[route("/books/:book_id")]
    Book { book_id: String },

    #[route("/books/:book_id/audio")]
    Audio { book_id: String },

    #[route("/books/:book_id/text")]
    Text { book_id: String },

    #[route("/books/:book_id/edit")]
    BookEdit { book_id: String },

    #[route("/series/:series_id/edit")]
    SeriesEdit { series_id: String },

    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
#[component]
pub(crate) fn PageNotFound(route: Vec<String>) -> Element {
    let nav = navigator();

    nav.replace(Route::Library {});

    rsx! { p {} }
}

#[component]
fn app() -> Element {
    console_error_panic_hook::set_once();
    let load = login::restore_user_from_storage();

    let user = use_signal(|| load);
    console_error_panic_hook::set_once();

    use_context_provider(|| user);
    use_context_provider(|| Signal::new(domain::library::LibraryMode::Normal));

    return rsx! {

        div {
            style: "min-height: 100vh; margin: 0; overflow: visible;",
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
            LoginGuard {Router::<Route> {}}

        }
    };
}
