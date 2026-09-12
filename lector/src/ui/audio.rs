use std::collections::VecDeque;

use dioxus::prelude::*;
use futures_util::stream::SplitSink;
use futures_util::{Sink, SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

use crate::domain::login;
use crate::{domain, ui::components::{TopBar, TopBarEntry, card::Cover}, Route, assets};


#[derive(Debug, Deserialize,Serialize,  Clone, PartialEq)]
struct AudioHeader {
    id: String,
    chapter: i32,
    start_offset: i32,
    end_offset: i32,
}


#[derive(Debug)]
struct AudioChunk {
    header: AudioHeader,
    bytes: Vec<u8>,
}

type Sender = SplitSink<WebSocket, Message>;
const AUDIO_MIME_TYPE: &str = "audio/mpeg";

const AUDIO_ELEMENT_ID: &str = "chunk-audio-player";

// ---------------------------------------------------------------------
// Master component
// ---------------------------------------------------------------------

#[component]
pub fn Audio(book_id: String) -> Element {

    let status = use_signal(|| "Disconnected".to_string());
    let connected = use_signal(|| false);
    let show_extra = use_signal(|| false);
    let queue: Signal<VecDeque<AudioChunk>> = use_signal(VecDeque::new);
    let sender: Signal<Option<Sender>> = use_signal(|| None);
    let playing = use_signal(||true);
    let current_header: Signal<Option<AudioHeader>> = use_signal(|| None);
    let cover_path=domain::cover::create_cover_path(book_id.clone());


    let top_entries = vec![
        TopBarEntry { name: "Library".into(), path: Route::Library {} },
        TopBarEntry { name: "Book".into(), path: Route::Book { book_id: book_id.clone() } },
    ];



    rsx! {
        div {
            TopBar { entries: top_entries, show_extra: show_extra }

            WebSocketConnection {
                book_id: book_id.clone(),
                status,
                connected,
                queue,
                sender,
            }
             WakeLock { playing } 
            AudioPlayer {
                queue,
                playing,
                sender,
                current_header,
            }

            div {
                class: "flex flex-col items-center gap-4",
                Cover {
                    book_id: book_id.clone(),
                }
                PlayPauseButton { playing, current_header }
            }

            div {
                class: "mt-4",

                TimeBar {
                    book_id: book_id.clone(),
                    headers: current_header,
                }
            }

        }
    }
}

#[component]
fn PlayPauseButton(
    mut playing: Signal<bool>,
    mut current_header: Signal<Option<AudioHeader>>,
) -> Element {
    let loading = current_header().is_none();

    rsx! {
        div {
            class: "relative w-14 h-14 flex items-center justify-center",

            img {
                class: if loading {
                    "absolute inset-0 w-full h-full object-contain pointer-events-none select-none animate-spin"
                } else {
                    "absolute inset-0 w-full h-full object-contain pointer-events-none select-none"
                },
                src: if loading {
                    assets::LOADING
                } else if !playing() {
                    assets::PLAY
                } else {
                    assets::PAUSE
                }
            }

            button {
                class: "absolute inset-0 w-full h-full active:scale-90",
                onclick: move |_| playing.set(!playing()),
            }
        }
    }
}
// ---------------------------------------------------------------------
// WebSocket connection (headless - manages the connect/receive loop)
// ---------------------------------------------------------------------

#[component]
fn WebSocketConnection(
    book_id: String,
    mut status: Signal<String>,
    mut connected: Signal<bool>,
    mut queue: Signal<VecDeque<AudioChunk>>,
    mut sender: Signal<Option<Sender>>,
) -> Element {
    use_effect(move || {
        let book_id = book_id.clone();

        spawn(async move {
            let mut backoff_ms: u32 = 500;
            const MAX_BACKOFF_MS: u32 = 15_000;

            loop {
                let _ = login::refresh_auth().await;
                if domain::login::current_auth().is_none() {
                    status.set("Session expired".into());
                    connected.set(false);
                    sender.set(None);
                    break; // stop retrying; login guard takes it from here
                }


                dioxus::logger::tracing::info!("Connecting websocket for {book_id}");
                status.set("Connecting...".into());

                let mut ws = match WebSocket::open(&format!("/api/v1/audio/{}", book_id)) {
                    Ok(ws) => ws,
                    Err(e) => {
                        status.set(format!("Connection failed: {e:?}"));
                        gloo_timers::future::TimeoutFuture::new(backoff_ms).await;
                        backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                        continue;
                    }
                };


                connected.set(true);
                status.set("Connected".into());
                let _ = ws
                    .send(Message::Text(
                        json!({"type": "auth", "token": &domain::login::current_auth()}).to_string(),
                    ))
                    .await;
                let (tx, mut rx) = ws.split();
                sender.set(Some(tx));

                // Reset backoff once we're actually connected.
                backoff_ms = 500;
                let mut pending_header: Option<AudioHeader> = None;

                while let Some(msg) = rx.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<AudioHeader>(&text) {
                                Ok(header) => pending_header = Some(header),
                                Err(e) => {
                                    dioxus::logger::tracing::warn!(
                                        "Bad audio header, dropping: {e:?} ({text})"
                                    );
                                    pending_header = None;
                                }
                            }
                        }
                        Ok(Message::Bytes(bytes)) => {
                            dioxus::logger::tracing::info!("Received {} bytes", bytes.len());
                            match pending_header.take() {
                                Some(header) => {
                                    queue.write().push_back(AudioChunk { header, bytes });
                                }
                                None => {
                                    dioxus::logger::tracing::warn!(
                                        "Got {} bytes with no preceding header, dropping",
                                        bytes.len()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            status.set(format!("Socket error: {e:?}"));
                            break;
                        }
                    }
                }

                connected.set(false);
                sender.set(None);
                status.set("Reconnecting...".into());
                gloo_timers::future::TimeoutFuture::new(backoff_ms).await;
                backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
            }
        });
    });

    rsx! {}
}

fn audio_element() -> Option<HtmlAudioElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id(AUDIO_ELEMENT_ID)?
        .dyn_into::<HtmlAudioElement>()
        .ok()
}

/// Resolves once the given event fires on `target`. Used to await the
/// <audio> element's "ended" event without blocking the JS event loop.
async fn wait_for_event_once(target: &HtmlAudioElement, event: &str) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let closure = Closure::once(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        // `add_event_listener_with_callback` is inherited from EventTarget
        // via HtmlAudioElement's Deref chain.
        let _ = target.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
        // Closure::once frees itself after firing, so leaking the wrapper
        // here is safe and intentional.
        closure.forget();
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

fn bytes_to_object_url(bytes: &[u8]) -> Result<String, JsValue> {
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array);

    let opts = BlobPropertyBag::new();
    opts.set_type(AUDIO_MIME_TYPE);

    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts)?;
    Url::create_object_url_with_blob(&blob)
}

#[component]
fn AudioPlayer(
    mut queue: Signal<VecDeque<AudioChunk>>,
    playing: Signal<bool>,
    mut sender: Signal<Option<Sender>>,
    mut current_header: Signal<Option<AudioHeader>>,
) -> Element {
    use_effect(move || {
        let want_playing = playing();
        if let Some(audio) = audio_element() {
            if want_playing {
                let _ = audio.play();
            } else {
                let _ = audio.pause();
            }
        }
    });

    use_effect(move || {
        spawn(async move {
            let mut active_object_url: Option<String> = None;

            loop {
                // Wait until there's something to play.
                while queue.read().is_empty() {
                    gloo_timers::future::TimeoutFuture::new(100).await;
                }

                let Some(chunk) = queue.write().pop_front() else {
                    continue;
                };

                let Some(audio) = audio_element() else {
                    // <audio> not mounted yet somehow - put the chunk
                    // back and retry shortly rather than dropping it.
                    queue.write().push_front(chunk);
                    gloo_timers::future::TimeoutFuture::new(50).await;
                    continue;
                };

                let url = match bytes_to_object_url(&chunk.bytes) {
                    Ok(url) => url,
                    Err(e) => {
                        dioxus::logger::tracing::error!("Failed to build audio blob: {e:?}");
                        continue;
                    }
                };

                if let Some(old) = active_object_url.take() {
                    Url::revoke_object_url(&old).ok();
                }

                audio.set_src(&url);
                active_object_url = Some(url);
                current_header.set(Some(chunk.header.clone()));

                if let Some(tx) = sender.write().as_mut() {
                    if let Ok(payload) = serde_json::to_string(&chunk.header) {
                        let _ = tx.send(Message::Text(payload)).await;
                    }
                }

                if playing() {
                    let _ = audio.play();
                }
                wait_for_event_once(&audio, "ended").await;
            }
        });
    });

    rsx! {
        audio { id: AUDIO_ELEMENT_ID }
    }
}

use crate::infra::book::fetch_book_progress;
#[component]
pub fn TimeBar(book_id: String, headers: Signal<Option<AudioHeader>>) -> Element {

    let mut progress: Signal<f64> = use_signal(||0.0);

    use_effect(move || {
        let b=book_id.clone();
        if headers().is_none(){
            return
        }

        spawn(async move{
            match fetch_book_progress(&b).await{
                Err(_)=>{},
                Ok(value)=>{ progress.set(value.progress); }
            }
        });
    });

    rsx! {
        div {
            class: "w-full flex flex-col items-center",
            div {
                class: "w-[90%] relative",
                div {
                    class: "bg-gray-300 h-3 rounded-full overflow-hidden w-full",
                    div {
                        class: "h-full bg-blue-500 transition-all duration-300 rounded-full",
                        style: "width: {(progress() * 100.0).round()}%;",
                    }
                }
                p {
                    class: "absolute left-0 -bottom-6 text-sm",
                    "{(progress() * 100.0).round()}%"
                }
            }
        }
    }
}


// ---------------------------------------------------------------------
// Screen wake lock (keeps the device from sleeping while audio plays)
// ---------------------------------------------------------------------

/// Requests a screen wake lock via `navigator.wakeLock.request("screen")`.
/// Uses `js_sys::Reflect` instead of typed `web_sys::WakeLock` bindings so
/// we don't need the "WakeLock"/"WakeLockSentinel" web-sys features just
/// for this. Returns the sentinel (needed to release the lock later), or
/// `None` if the API isn't available or the request failed.
async fn request_wake_lock() -> Option<JsValue> {
    let navigator = web_sys::window()?.navigator();
    let wake_lock = js_sys::Reflect::get(&navigator, &JsValue::from_str("wakeLock")).ok()?;
    if wake_lock.is_undefined() || wake_lock.is_null() {
        return None;
    }
    let request_fn = js_sys::Reflect::get(&wake_lock, &JsValue::from_str("request")).ok()?;
    let request_fn: js_sys::Function = request_fn.dyn_into().ok()?;
    let promise = request_fn.call1(&wake_lock, &JsValue::from_str("screen")).ok()?;
    let promise: js_sys::Promise = promise.dyn_into().ok()?;
    wasm_bindgen_futures::JsFuture::from(promise).await.ok()
}

/// Releases a sentinel obtained from `request_wake_lock`. Fire-and-forget.
fn release_wake_lock(sentinel: &JsValue) {
    if let Ok(release_fn) = js_sys::Reflect::get(sentinel, &JsValue::from_str("release")) {
        if let Ok(release_fn) = release_fn.dyn_into::<js_sys::Function>() {
            let _ = release_fn.call0(sentinel);
        }
    }
}

fn document_visible() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| js_sys::Reflect::get(&d, &JsValue::from_str("visibilityState")).ok())
        .and_then(|v| v.as_string())
        .map(|s| s == "visible")
        .unwrap_or(true)
}
use crate::dioxus_core::use_drop;
/// Headless component that holds a screen wake lock for as long as
/// `playing` is true, and re-acquires it when the page becomes visible
/// again (the browser force-releases it whenever the tab is hidden).
#[component]
fn WakeLock(playing: Signal<bool>) -> Element {
    let mut sentinel: Signal<Option<JsValue>> = use_signal(|| None);

    use_effect(move || {
        let want_lock = playing();
        spawn(async move {
            if want_lock {
                if sentinel.read().is_none() {
                    if let Some(s) = request_wake_lock().await {
                        sentinel.set(Some(s));
                    }
                }
            } else if let Some(s) = sentinel.write().take() {
                release_wake_lock(&s);
            }
        });
    });

    use_effect(move || {
        let closure = Closure::wrap(Box::new(move || {
            if document_visible() && playing() && sentinel.read().is_none() {
                spawn(async move {
                    if sentinel.read().is_none() {
                        if let Some(s) = request_wake_lock().await {
                            sentinel.set(Some(s));
                        }
                    }
                });
            }
        }) as Box<dyn Fn()>);

        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let _ = doc.add_event_listener_with_callback(
                "visibilitychange",
                closure.as_ref().unchecked_ref(),
            );
        }
        // Leaked intentionally, same rationale as the "ended" listener above.
        closure.forget();
    });

    use_drop(move || {
        if let Some(s) = sentinel.write().take() {
            release_wake_lock(&s);
        }
    });

    rsx! {}
}