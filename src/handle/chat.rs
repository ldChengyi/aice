//! 文本聊天接口；内部错误写入日志，对客户端返回简洁提示。

use crate::{
    init::AppState,
    logic::chat::{self, ChatError},
};
use axum::{extract::State, http::StatusCode};

/// 接收纯文本请求；空输入、未配置模型和执行失败分别返回 400、503、502。
pub(crate) async fn reply(
    State(state): State<AppState>,
    body: String,
) -> Result<String, (StatusCode, &'static str)> {
    chat::reply(state.chat_agent.as_deref(), &body)
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
