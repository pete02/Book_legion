use dioxus::prelude::*;

mod home;
pub use home::Home;

mod audioview;
pub use audioview::AudioView;

mod readview;
pub use readview::ReadView;

mod bookview;
pub use bookview::BookView;

use crate::models::GlobalState;



#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},

    #[route("/AudioView")]
    #[redirect("/AudioView", || {
        let global = use_context::<Signal<GlobalState>>();
        if global().book.is_none() {
            Route::BookView {}
        } else {
            Route::AudioView {}
        }
    })]
    AudioView { },

    #[route("/ReadView")]
    #[redirect("/ReadView", || {
        let global = use_context::<Signal<GlobalState>>();
        if global().book.is_none() {
            Route::BookView {}
        } else {
            Route::ReadView {}
        }
    })]
    ReadView { },

    #[route("/BookView")]
    BookView { },
}

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            id: "nav-bar",
            style: "display: flex; flex-direction: column; height: 100vh;",
            div {
                id: "navbar",
                class: "h-16 flex items-center bg-gray-200 dark:bg-gray-800 px-4", 
                Link {
                    to: Route::Home {},
                    "Home"
                }
                Link {
                    to: Route::AudioView {  },
                    "AudioView"
                }

                Link {
                    to: Route::ReadView {  },
                    "ReadView"
                }
                Link {
                    to: Route::BookView {  },
                    "BookView"
                }
            }

            div {
                id: "book-container",
                style: "flex: 1; overflow: hidden;", // Outlet takes the remaining space exactly
                Outlet::<Route> { key: current_route.clone() }
            }
         }
    }
}
