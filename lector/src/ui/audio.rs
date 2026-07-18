use serde::{Deserialize, Serialize};
use gloo_net::websocket::futures::WebSocket;

use futures_util::{SinkExt, StreamExt};

#[derive(Debug, Serialize)]
struct CursorMessage {
    chapter: i32,
    index: i32,
}

#[derive(Debug, Deserialize, Clone)]
struct AudioHeader {
    id: String,
    chapter: i32,
    start_offset: i32,
    end_offset: i32,
}

use dioxus::prelude::*;

use crate::domain;

#[component]
pub fn Audio(book_id: String)->Element{
let mut status = use_signal(|| "Disconnected".to_string());
    let mut last_header = use_signal(|| None::<AudioHeader>);
    let mut binary_count = use_signal(|| 0usize);
    let mut connected = use_signal(|| false);

    let mut sender = use_signal(|| None::<futures_util::stream::SplitSink<
        WebSocket,
        gloo_net::websocket::Message,
    >>);

    use_effect(move || {
        dioxus::logger::tracing::info!("Effect started");
        let token = domain::login::current_auth().unwrap_or_default();

        let bookd_copied=book_id.clone();
        spawn(async move {
            dioxus::logger::tracing::info!("Spawn started");
            status.set("Connecting...".into());
            dioxus::logger::tracing::info!("Status set");

            let ws = match WebSocket::open(
                &format!("/api/v1/audio/{}?token={}", bookd_copied, token),
            ) {
                Ok(ws) => ws,
                Err(e) => {
                    status.set(format!("Connection failed: {e:?}"));
                    return;
                }
            };
            
            connected.set(true);
            status.set("Connected".into());

            let (tx, mut rx) = ws.split();

            sender.set(Some(tx));

            let mut expecting_binary = false;

            while let Some(msg) = rx.next().await {

                match msg {

                    Ok(gloo_net::websocket::Message::Text(text)) => {

                        dioxus::logger::tracing::info!("Header: {}", text);

                        let header: AudioHeader =
                            serde_json::from_str(&text).unwrap();

                        last_header.set(Some(header));

                        expecting_binary = true;
                    }

                    Ok(gloo_net::websocket::Message::Bytes(bytes)) => {

                        dioxus::logger::tracing::info!("Received {} bytes", bytes.len());

                        if expecting_binary {

                            binary_count += 1;

                            expecting_binary = false;
                        }
                    }

                    Err(e) => {

                        status.set(format!("Socket error: {e:?}"));

                        break;
                    }
                }
            }

            connected.set(false);
            status.set("Disconnected again".into());
        });

    });

    rsx! {

        div {

            h2 { "Audio websocket demo" }

            p { "Status: {status}" }

            p {
                "Connected: "
                if connected() { "yes" } else { "no" }
            }

            p {
                "Binary chunks received: {binary_count}"
            }

            if let Some(header) = last_header() {

                div {

                    p { "Last chunk:" }

                    p { "ID: {header.id}" }

                    p { "Chapter: {header.chapter}" }

                    p {
                        "{header.start_offset} -> {header.end_offset}"
                    }

                }
            }

        }

    }
}




