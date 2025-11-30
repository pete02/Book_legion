use axum::{
    routing::{get,post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

mod test;
mod book_handler;
mod models;

// Import your core logic here:
mod logic;
use logic::*;

#[derive(Clone)]
struct AppState {
    manifest: String,
    prefix: String,
}
impl AppState {
    fn path(&self) -> String {
        format!("{}/{}", self.prefix, self.manifest)
    }
}
#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        manifest: "books.json".to_string(),
        prefix: "./data".to_string(),
    });

    let app = Router::new()
        .route("/init", get(init_handler))
        .route("/book", post(book_handler))
        .route("/audio", post(audio_handler))
        .route("/audiomap",post(audiomap))
        .route("/update", post(update_handler))
        .route("/manifest", get(manifest_handler))
        .route("/cover/{book}", get(cover_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("🚀 Server running on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
