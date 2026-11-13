use async_trait::async_trait;
use std::error::Error;

/// Single-turn text generation used by agent workflows.
#[async_trait]
pub trait AgentModel: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
}
