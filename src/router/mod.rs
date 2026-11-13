use crate::{handle, init::AppState};
use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};

pub fn build(state: AppState) -> Router {
    let frontend = Router::new()
        .route("/personas", get(handle::personas::page))
        .route("/assets/personas.css", get(handle::personas::stylesheet));

    Router::new()
        .route("/healthz", get(handle::health::check))
        .route("/chat", post(handle::chat::reply))
        .route("/ui", get(|| async { Redirect::temporary("/ui/personas") }))
        .merge(frontend.clone())
        .nest("/ui", frontend)
        .with_state(state)
}
