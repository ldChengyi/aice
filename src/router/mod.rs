//! 集中注册 HTTP 路由，并注入应用共享状态。

use crate::{handle, init::AppState};
use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};

/// 组装 API 和页面路由；页面与下载同时保留根路径及 /ui 前缀入口。
pub fn build(state: AppState) -> Router {
    // /ui 路由组与 Nginx 的公开转发范围对应，包含页面、样式和导出下载。
    let frontend = Router::new()
        .route(
            "/character-cards/exports/{id}/character-card.xlsx",
            get(handle::character_card::download),
        )
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
