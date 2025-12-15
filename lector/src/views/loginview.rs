use dioxus::{logger::tracing, prelude::*};
use crate::{components::{server_api::fetch_login}, models::GlobalState, views::Route};

#[component]
pub fn LoginView() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let mut global = use_context::<Signal<GlobalState>>();
    let navigator = use_navigator(); // <- navigator hook for redirects


    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 100vh;
                padding: 24px;
            ",

            h1 { "Login" }

            div {
                style: "
                    display: flex;
                    flex-direction: column;
                    gap: 12px;
                    width: 300px;
                ",

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
                    class: "
                        bg-blue-600
                        hover:bg-blue-700
                        active:bg-blue-800
                        text-white
                        font-semibold
                        py-2
                        px-4
                        rounded-lg
                        transition-colors
                        duration-150
                        disabled:opacity-50
                        disabled:cursor-not-allowed
                    ",
                    disabled: loading(),
                    onclick: move |_| {
                        let user = username();
                        let pass = password();

                        loading.set(true);
                        error.set(None);

                        let navigator = navigator.clone();

                        spawn(async move {
                            match fetch_login(&user, &pass).await {
                                Ok(tokens) => {
                                    // Update global state
                                    let expiry = chrono::Utc::now() + chrono::Duration::minutes(5);
                                    let name_clone = user.clone();
                                    let access_token = tokens.access_token.clone();
                                    let refresh_token = tokens.refresh_token.clone();
                                    tracing::debug!("set global with corerct expiry");
                                    global.with_mut(|state| {
                                        state.user = Some(name_clone.clone());
                                        state.access_token = Some(access_token);
                                        state.refresh_token = Some(refresh_token);
                                        state.token_expiry=Some(expiry);
                                    });

                                    // Redirect to `/`
                                    navigator.replace(Route::LibraryView {}); // assuming LibraryView is at `/`
                                }
                                Err(e) => {
                                    error.set(Some(e.to_string()));
                                }
                            }
                            loading.set(false);
                        });
                    },
                    if loading() { "Logging in…" } else { "Login" }
                }

                if let Some(err) = error() {
                    div {
                        style: "color: red; font-size: 0.9em;",
                        "{err}"
                    }
                }
            }
        }
    }
}
