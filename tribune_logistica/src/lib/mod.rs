use axum::{
    routing::{get,post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

pub mod book_handler;
pub mod models;
pub mod audio_handler;
pub mod password_handler;

pub mod db_handlers;
pub mod update_handler;
use tokio::sync::{RwLock, mpsc};
pub mod audio_gen_handler;


// Import your core logic here:
mod logic;


use logic::*;
use crate::{buffer_handler::FillerCommand, models::AudioBuffer, password_handler::generate_secret};

pub mod buffer_handler;

#[derive(Clone)]
struct AppState {
    manifest: String,
    prefix: String,
    config: String,
    secret: [u8; 32],
    global_buffer: Arc<RwLock<Option<Arc<RwLock<AudioBuffer>>>>>,
    filler_tx: Arc<RwLock<Option<mpsc::Sender<FillerCommand>>>>,
}

//        .route("/audiomap",post(audiomap))


pub async fn server()->() {
    let state = Arc::new(AppState {
        manifest: "books.json".to_string(),
        prefix: "./data".to_string(),
        config: "./config".to_string(),
        secret: generate_secret(),
    global_buffer: Arc::new(RwLock::new(Some(Arc::new(RwLock::new(AudioBuffer::new(20, 30)))))),
    filler_tx: Arc::new(RwLock::new(None)),
    });

    let app = Router::new()
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
        .route("/init", get(init_handler))
        .route("/book", post(book_handler))
        .route("/audio", post(audio_endpoint))
        .route("/update", post(update_endpoint))
        .route("/manifest", get(manifest_handler))
        .route("/cover/{book}", get(cover_handler))
        .route("/css/{book}", get(css_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Server running on http://{}", addr);
    return axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutdown signal received");
}
