//! 模型适配接口：只负责一次补全，工具循环交给业务层 Agent。

use async_trait::async_trait;
use rig_core::completion::{CompletionRequest, CompletionResponse};
use std::error::Error;

/// 可以跨异步任务传播的模型调用错误。
pub type ModelError = Box<dyn Error + Send + Sync>;

/// 统一的单次模型补全接口；Agent 负责工具执行与消息上下文管理。
#[async_trait]
pub trait AgentModel: Send + Sync {
    /// 提交包含历史消息及工具定义的请求，返回原始补全响应或调用错误。
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError>;
}
