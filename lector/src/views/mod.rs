use dioxus::prelude::*;

mod home;
pub use home::Home;

mod audioview;
pub use audioview::AudioView;



#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/AudioView")]
    AudioView { },
}

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::AudioView {  },
                "AudioView"
            }
        }

        Outlet::<Route> {
            key: current_route.clone()
        }
    }
}
