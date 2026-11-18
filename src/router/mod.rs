use crate::{handle, init::AppState};
use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};

pub fn build(state: AppState) -> Router {
    let frontend = Router::new()
        .route(
            "/character-cards/exports/{id}",
            get(handle::character_card::download),
        )
        .route("/personas", get(handle::personas::page))
        .route("/assets/personas.css", get(handle::personas::stylesheet));

    Router::new()
        .route("/healthz", get(handle::health::check))
        .route("/chat", post(handle::chat::reply))
        .route(
            "/character-cards/default",
            get(handle::character_card::default_card),
        )
        .route(
            "/character-cards/templates",
            get(handle::character_card::templates),
        )
        .route(
            "/character-cards/templates/{id}",
            get(handle::character_card::template_card),
        )
        .route(
            "/character-cards/templates/{id}/fields",
            get(handle::character_card::fields),
        )
        .route(
            "/character-cards/templates/{id}/export",
            post(handle::character_card::export),
        )
        .route("/ui", get(|| async { Redirect::temporary("/ui/personas") }))
        .merge(frontend.clone())
        .nest("/ui", frontend)
        .with_state(state)
}
