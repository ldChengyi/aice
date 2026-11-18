use super::agent::ChatAgent;
use std::error::Error;

pub enum ChatError {
    EmptyPrompt,
    ModelUnavailable,
    ModelFailed(Box<dyn Error + Send + Sync>),
}

pub async fn reply(agent: Option<&ChatAgent>, prompt: &str) -> Result<String, ChatError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(ChatError::EmptyPrompt);
    }

    let agent = agent.ok_or(ChatError::ModelUnavailable)?;
    agent.reply(prompt).await.map_err(ChatError::ModelFailed)
}
