use async_trait::async_trait;
use rig_core::completion::{CompletionRequest, CompletionResponse};
use std::error::Error;

pub type ModelError = Box<dyn Error + Send + Sync>;

/// One model completion. The Agent owns tool execution and conversation flow.
#[async_trait]
pub trait AgentModel: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError>;
}
