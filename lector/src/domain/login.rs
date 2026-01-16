use dioxus::{logger::tracing, prelude::*};
use crate::infra::login;



pub struct User {
    pub username: String,
    pub refresh_token: Option<String>,
    pub auth_token: Option<String>,
}

pub fn restore_user_from_storage() -> User {
    let refresh_token = web_sys::window()
        .unwrap()
        .session_storage()
        .unwrap()
        .unwrap()
        .get_item("refresh_token")
        .ok()
        .flatten();

    let auth_token = web_sys::window()
        .unwrap()
        .session_storage()
        .unwrap()
        .unwrap()
        .get_item("auth_token")
        .ok()
        .flatten();
    tracing::debug!("Got refresh: {:?}",refresh_token);
    tracing::debug!("Got auth: {:?}",auth_token);

    User {
        username: "".into(),
        refresh_token,
        auth_token,
    }
}

pub fn persist_user(user: &User) {
    let storage = web_sys::window().unwrap().session_storage().unwrap().unwrap();
    if let Some(rt) = &user.refresh_token {
        storage.set_item("refresh_token", rt).unwrap();
    }
    if let Some(at) = &user.auth_token {
        storage.set_item("auth_token", at).unwrap();
    }
}



pub fn attempt_login(username: String, password: String, error:Signal<String>, loading:Signal<bool>) {
    spawn({
        let mut user = use_context::<Signal<User>>();
        let mut error = error.clone();
        let mut loading=loading.clone();
        async move {
            loading.set(true);
            match login::login(&username, &password).await {
                Ok(resp) => {
                    let u=user.clone();
                    let new_user=User{
                        username: u.read().username.clone(),
                        auth_token: Some(resp.auth_token),
                        refresh_token: Some(resp.refresh_token)
                    };
                    loading.set(false);
                    persist_user(&new_user);
                    user.set(new_user);
                    error.set("".to_owned());
                }
                Err(e) => {
                    loading.set(false);
                    error.set(e.to_string());
                }
            }
        }
    });
}