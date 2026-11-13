use crate::{handle, init::AppState};
use axum::{
    Router,
    routing::{get, post},
};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(handle::health::check))
        .route("/chat", post(handle::chat::reply))
        .route("/personas", get(handle::personas::page))
        .route("/assets/personas.css", get(handle::personas::stylesheet))
        .with_state(state)
}
