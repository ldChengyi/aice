use crate::{
    config::Config,
    infra::llm::build_model,
    service::{agent::AgentModel, database},
};
use sqlx::PgPool;
use std::{io, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub database: Option<PgPool>,
    pub agent_model: Option<Arc<dyn AgentModel>>,
}

impl AppState {
    pub async fn new(config: &Config) -> io::Result<Self> {
        let agent_model = config
            .llm
            .as_ref()
            .map(build_model)
            .transpose()
            .map_err(io::Error::other)?
            .map(Arc::from);
        let database = database::connect().await?;

        Ok(Self {
            database,
            agent_model,
        })
    }
}
