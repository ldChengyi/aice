use crate::{
    config::{LlmConfig, LlmProvider},
    service::agent::{AgentModel, ModelError},
};
use async_trait::async_trait;
use rig_core::{
    Model,
    completion::{CompletionRequest, CompletionResponse},
    providers::openai::{
        OpenAIConfig,
        wire::{Chat, DEEPSEEK},
    },
};
use std::env;

pub fn build_model(config: &LlmConfig) -> Result<Box<dyn AgentModel>, ModelError> {
    match config.provider {
        LlmProvider::Deepseek => Ok(Box::new(DeepSeekModel::new(config)?)),
    }
}

struct DeepSeekModel {
    model: Model<Chat>,
}

impl DeepSeekModel {
    fn new(config: &LlmConfig) -> Result<Self, ModelError> {
        let api_key = env::var("DEEPSEEK_API_KEY")
            .map_err(|_| "set DEEPSEEK_API_KEY before calling DeepSeek")?;
        if api_key.trim().is_empty() {
            return Err("DEEPSEEK_API_KEY is empty".into());
        }

        let client = OpenAIConfig::with_key(&DEEPSEEK, api_key)
            .with_base_url(&config.base_url)
            .client();
        Ok(Self {
            model: client.chat(&config.model),
        })
    }
}

#[async_trait]
impl AgentModel for DeepSeekModel {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let response = self.model.call(request).await?;
        if let Some(error) = &response.error {
            return Err(format!("DeepSeek completion failed: {error}").into());
        }
        Ok(response)
    }
}
