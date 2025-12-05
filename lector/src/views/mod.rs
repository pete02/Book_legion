use dioxus::prelude::*;

mod home;
pub use home::Home;

mod audioview;
pub use audioview::AudioView;

mod bookview;
pub use bookview::BookView;



#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/AudioView")]
    AudioView { },
    #[route("/BookView")]
    BookView { },
}

#[component]
pub fn Navbar() -> Element {
    rsx! {
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
                to: Route::BookView {  },
                "BookView"
            }
        }

        Outlet::<Route> {
            key: current_route.clone()
        }
    }
}
