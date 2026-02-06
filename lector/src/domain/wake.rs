use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
let wakeLock = null;

export async function requestWakeLock() {
    if ('wakeLock' in navigator) {
        try {
            wakeLock = await navigator.wakeLock.request('screen');
        } catch (e) {
            console.warn('WakeLock failed:', e);
        }
    }
}

export function releaseWakeLock() {
    if (wakeLock) {
        wakeLock.release();
        wakeLock = null;
    }
}
"#)]
extern "C" {
    async fn requestWakeLock();
    fn releaseWakeLock();
}

use wasm_bindgen_futures::spawn_local;

pub fn on_audio_play() {
    spawn_local(async {
        requestWakeLock().await;
    });
}

pub fn on_audio_pause() {
    releaseWakeLock();
}
