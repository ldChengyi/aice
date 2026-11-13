use crate::service::agent::AgentModel;
use std::error::Error;

pub enum ChatError {
    EmptyPrompt,
    ModelUnavailable,
    ModelFailed(Box<dyn Error + Send + Sync>),
}

pub async fn reply(model: Option<&dyn AgentModel>, prompt: &str) -> Result<String, ChatError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(ChatError::EmptyPrompt);
    }

    let model = model.ok_or(ChatError::ModelUnavailable)?;
    model.complete(prompt).await.map_err(ChatError::ModelFailed)
}
