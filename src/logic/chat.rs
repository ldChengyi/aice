//! 聊天入口校验，并将 Agent 执行结果转换为业务错误。

use super::agent::ChatAgent;
use std::error::Error;

/// 聊天入口的业务错误，交由 HTTP 层映射为对应状态码。
pub enum ChatError {
    EmptyPrompt,
    ModelUnavailable,
    ModelFailed(Box<dyn Error + Send + Sync>),
}

/// 先检查空输入和 Agent 可用性，再执行一次独立的聊天请求。
pub async fn reply(agent: Option<&ChatAgent>, prompt: &str) -> Result<String, ChatError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(ChatError::EmptyPrompt);
    }

    let agent = agent.ok_or(ChatError::ModelUnavailable)?;
    agent.reply(prompt).await.map_err(ChatError::ModelFailed)
}
