use dioxus::prelude::*;
use crate::domain::{self, login::User};
use crate::styles;

#[component]
pub fn LoginGuard(children: Element) -> Element {
    let user: Signal<User> = use_context::<Signal<User>>();

    rsx! {
        if user.read().refresh_token.is_none() {
            Login {}
        } else {
            {children}
        }
    }
}

#[component]
fn Login() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| "".to_owned());
    let loading = use_signal(|| false);

    rsx! {
        div { style: styles::LOGIN_CONTAINER,
            h1 { "Login" }

            div { style: styles::LOGIN_FORM,
                input {
                    r#type: "text",
                    placeholder: "Username",
                    value: "{username()}",
                    oninput: move |e| username.set(e.value()),
                }
                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password()}",
                    oninput: move |e| password.set(e.value()),
                }

                button {
                    class: styles::LOGIN_BUTTON,
                    disabled: loading(),
                    onclick: move |_| {
                        error.set("".to_owned());
                        domain::login::attempt_login(username(), password(), error,loading);
                    },
                    if loading() { "Logging in…" } else { "Login" }
                }

                div { style: styles::LOGIN_ERROR, "{error}" }
  
            }
        }
    }
}