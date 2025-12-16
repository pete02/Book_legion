use std::time::Duration;

use dioxus::{core::spawn, hooks::{use_context, use_effect}, logger::tracing, signals::{Signal, WritableExt}};
use dioxus::prelude::*;
use gloo_timers::future::sleep;

use crate::{components::server_api::refresh_access_token, models::GlobalState}; // lightweight async sleep

#[component]
pub fn AccessTokenHook() ->Element{
    let mut started=use_signal(||false);
    let mut global=use_context::<Signal<GlobalState>>();
    use_effect(move || {
        if started() {return;};
        started.set(true);
        spawn(async move{
                loop {
                    sleep(Duration::from_secs(10)).await;
                    let Some(expiry_time)=global().token_expiry.clone() else {continue;};
                    if chrono::Utc::now() >= expiry_time- chrono::Duration::seconds(30) {
                        let Some(user)=global().user.clone() else {continue;};
                        let Some(refresh_token)=global().refresh_token.clone() else {continue;};
                        match refresh_access_token(user, refresh_token).await {
                            Err(_)=>tracing::error!("could not refresh tokens"),
                            Ok(tokens)=>{
                                tracing::debug!("refrehs tokens");
                                global.with_mut(|state|{
                                    state.access_token=Some(tokens.access_token.clone());
                                    state.refresh_token=Some(tokens.refresh_token.clone());
                                    state.token_expiry=Some(chrono::Utc::now() + chrono::Duration::minutes(5));
                                })
                            }
                        }

                    }
                    
                }
            });
        });
    rsx!(div {})
}
