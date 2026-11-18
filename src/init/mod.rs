use crate::{config::Config, infra::llm::build_model, logic::agent::ChatAgent, service::database};
use character_card_mcp::CardExports;
use sqlx::PgPool;
use std::{io, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub database: Option<PgPool>,
    pub chat_agent: Option<Arc<ChatAgent>>,
    pub card_exports: Arc<CardExports>,
}

impl AppState {
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
