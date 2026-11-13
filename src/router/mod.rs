mod health;

use crate::{
    init::AppState,
    logic::chat::{self as chat_logic, ChatError},
};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::check))
        .route("/chat", post(chat))
        .with_state(state)
}

async fn chat(
    State(state): State<AppState>,
    body: String,
) -> Result<String, (StatusCode, &'static str)> {
    chat_logic::reply(state.agent_model.as_deref(), &body)
        .await
        .map_err(|error| match error {
            ChatError::EmptyPrompt => (StatusCode::BAD_REQUEST, "请输入问题。"),
            ChatError::ModelUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Aice 尚未配置聊天模型。")
            }
            ChatError::ModelFailed(error) => {
                eprintln!("chat request failed: {error}");
                (StatusCode::BAD_GATEWAY, "模型请求失败，请稍后再试。")
            }
        })
}
