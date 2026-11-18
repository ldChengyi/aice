//! DeepSeek 模型适配：读取密钥、构建客户端并执行单次补全。

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

/// 按供应商配置构建模型；缺少有效密钥时返回错误。
pub fn build_model(config: &LlmConfig) -> Result<Box<dyn AgentModel>, ModelError> {
    match config.provider {
        LlmProvider::Deepseek => Ok(Box::new(DeepSeekModel::new(config)?)),
    }
}

/// 保存可复用的客户端模型句柄，不保存用户对话状态。
struct DeepSeekModel {
    model: Model<Chat>,
}

impl DeepSeekModel {
    /// 从环境变量读取密钥，并绑定配置中的 API 地址与模型名。
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
    /// 保留工具调用和原始消息；整体超时由 Agent 或命令行调用方控制。
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let response = self.model.call(request).await?;
        if let Some(error) = &response.error {
            return Err(format!("DeepSeek completion failed: {error}").into());
        }
        Ok(response)
    }
}
