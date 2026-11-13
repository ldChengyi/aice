mod health;

use crate::service::agent::AgentModel;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;

pub fn build(model: Option<Arc<dyn AgentModel>>) -> Router {
    Router::new()
        .route("/healthz", get(health::check))
        .route("/chat", post(chat))
        .with_state(model)
}

async fn chat(
    State(model): State<Option<Arc<dyn AgentModel>>>,
    body: String,
) -> Result<String, (StatusCode, &'static str)> {
    let prompt = body.trim();
    if prompt.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "请输入问题。"));
    }

    let model = model.ok_or((StatusCode::SERVICE_UNAVAILABLE, "Aice 尚未配置聊天模型。"))?;
    model.complete(prompt).await.map_err(|error| {
        eprintln!("chat request failed: {error}");
        (StatusCode::BAD_GATEWAY, "模型请求失败，请稍后再试。")
    })
}
