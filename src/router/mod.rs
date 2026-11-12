mod health;

use axum::{
    Router,
    body::Bytes,
    routing::{get, post},
};

pub fn build() -> Router {
    Router::new()
        .route("/healthz", get(health::check))
        .route("/callback", post(callback))
}

async fn callback(body: Bytes) -> Bytes {
    body
}
