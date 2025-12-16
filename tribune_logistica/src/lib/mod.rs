use axum::{
    routing::{get,post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub mod book_handler;
pub mod models;

pub mod password_handler;

// Import your core logic here:
mod logic;
use logic::*;

use crate::password_handler::generate_secret;

#[derive(Clone)]
struct AppState {
    manifest: String,
    prefix: String,
    secret: [u8; 32]
}
impl AppState {
    fn path(&self) -> String {
        format!("{}/{}", self.prefix, self.manifest)
    }
}

pub async fn server()->() {
    let state = Arc::new(AppState {
        manifest: "books.json".to_string(),
        prefix: "/data".to_string(),
        secret: generate_secret()
    });

    let app = Router::new()
        .route("/api/login", post(login_handler))
        .route("/api/refresh", post(refresh_handler))
        .route("/api/init", get(init_handler))
        .route("/api/book", post(book_handler))
        .route("/api/audiomap",post(audiomap))
        .route("/api/audio", post(audio_handler))
        .route("/api/update", post(update_handler))
        .route("/api/manifest", get(manifest_handler))
        .route("/api/cover/{book}", get(cover_handler))
        .route("/api/css/{book}", get(css_handler))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .fallback_service(
            ServeDir::new("/public").fallback(ServeDir::new("/public/index.html"))
        );

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