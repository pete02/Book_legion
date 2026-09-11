use dioxus::prelude::*;
use crate::domain::{self, login::User};
use crate::styles;
use crate::infra::login;


#[component]
pub fn LoginGuard(children: Element) -> Element {
    let user: Signal<User> = use_context::<Signal<User>>();
    let user = user.read();

    if user.refresh_token.is_none() {
        rsx! {
            Login {}
        }
    } else if user.pin == "0000" {
        rsx! {
            ChangePinRequired {}
        }
    } else {
        rsx! {
            {children}
        }
    }
}

#[component]
fn ChangePinRequired() -> Element {
    let mut pin = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| false);

    rsx! {
        div { style: styles::LOGIN_CONTAINER,
            h1 { "Change your PIN" }

            div { style: styles::LOGIN_FORM,
                p { "You must change your PIN before continuing." }

                input {
                    r#type: "password",
                    placeholder: "New PIN",
                    value: "{pin()}",
                    oninput: move |e| pin.set(e.value()),
                }

                button {
                    class: styles::LOGIN_BUTTON,
                    disabled: loading(),
                    onclick: move |_| {
                        error.set(String::new());

                        let new_pin = pin();
                        if new_pin == "0000" {
                            error.set("Please choose a different PIN.".into());
                            return;
                        }

                        loading.set(true);

                        spawn(async move {
                            match login::change_pin(&new_pin).await {
                                Ok(()) => {
                                    loading.set(false);
                                }
                                Err(e) => {
                                    error.set(e);
                                    loading.set(false);
                                }
                            }
                        });
                    },

                    if loading() {
                        "Changing PIN…"
                    } else {
                        "Change PIN"
                    }
                }

                div {
                    style: styles::LOGIN_ERROR,
                    "{error}"
                }
            }
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