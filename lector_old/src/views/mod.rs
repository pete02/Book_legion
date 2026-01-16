use dioxus::prelude::*;


mod audioview;
pub use audioview::AudioView;

mod readview;
pub use readview::ReadView;

mod bookview;
pub use bookview::BookView;

mod library_view;
pub use library_view::LibraryView;

mod loginview;
pub use loginview::LoginView;

use crate::components::AccessTokenHook;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    LibraryView {},

    #[route("/LoginView")]
    LoginView { },

    #[route("/AudioView")]
    AudioView { },

    #[route("/ReadView")]
    ReadView { },

    #[route("/BookView")]
    BookView { },

}

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            id: "nav-bar",
            style: "display: flex; flex-direction: column; height: auto; min-height: 100vh; overflow: visible;",
            div {
                id: "book-container; overflow: visible;",
                style: "flex: 1;", // Outlet takes the remaining space exactly
                Outlet::<Route> { key: current_route.clone() }
                AccessTokenHook{}
            }
         }
    }
}
