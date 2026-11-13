use crate::{
    config::{LlmConfig, LlmProvider},
    service::agent::AgentModel,
};
use async_trait::async_trait;
use rig_core::{
    Model,
    completion::CompletionRequest,
    providers::openai::{
        OpenAIConfig,
        wire::{Chat, DEEPSEEK},
    },
};
use std::{env, error::Error, time::Duration};

pub fn build_model(
    config: &LlmConfig,
) -> Result<Box<dyn AgentModel>, Box<dyn Error + Send + Sync>> {
    match config.provider {
        LlmProvider::Deepseek => Ok(Box::new(DeepSeekModel::new(config)?)),
    }
}

struct DeepSeekModel {
    model: Model<Chat>,
}

impl DeepSeekModel {
    fn new(config: &LlmConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
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
    async fn complete(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response = tokio::time::timeout(
            Duration::from_secs(60),
            self.model.call(CompletionRequest::new(prompt)),
        )
        .await
        .map_err(|_| "DeepSeek request timed out after 60 seconds")??;

        let text = response.text();
        if text.trim().is_empty() {
            return Err("DeepSeek returned no text".into());
        }
        Ok(text)
    }
}
