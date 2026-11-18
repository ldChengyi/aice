//! 在启动阶段组装共享状态，供各 HTTP 请求复用。

use crate::{config::Config, infra::llm::build_model, logic::agent::ChatAgent, service::database};
use character_card_mcp::CardExports;
use sqlx::PgPool;
use std::{io, sync::Arc};

/// 由 Axum 共享的应用状态；Arc 是线程安全引用计数，克隆时共享服务而非复制服务。
#[derive(Clone)]
pub struct AppState {
    pub database: Option<PgPool>,
    pub chat_agent: Option<Arc<ChatAgent>>,
    pub card_exports: Arc<CardExports>,
}

impl AppState {
    /// 根据配置创建服务依赖；已启用的模型或数据库初始化失败会中止启动。
    pub async fn new(config: &Config) -> io::Result<Self> {
        let card_exports = Arc::new(CardExports::new(
            config.public_base_url.clone().unwrap_or_default(),
        ));
        let chat_agent = config
            .llm
            .as_ref()
            .map(build_model)
            .transpose()
            .map_err(io::Error::other)?
            .map(|model| Arc::new(ChatAgent::new(model, card_exports.clone())));
        let database = database::connect().await?;

        Ok(Self {
            database,
            chat_agent,
            card_exports,
        })
    }
}
