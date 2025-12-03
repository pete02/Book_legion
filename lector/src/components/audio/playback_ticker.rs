use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_timers::future::TimeoutFuture;


const TICK_INTERVAL_MS: u32 = 100;
const TICK_INCREMENT: f64 = TICK_INTERVAL_MS as f64 /1000.0;
pub fn use_playback_tick(playing: Signal<bool>, mut total_played: Signal<f64>) {
    use_effect(move || {
        spawn_local(async move {
            loop {
                TimeoutFuture::new(TICK_INTERVAL_MS).await;
                if *playing.read() {
                    total_played.with_mut(|t| *t += TICK_INCREMENT);
                }
            }
        });
    });
}
