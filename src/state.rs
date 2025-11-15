use crate::{config::Config, db::pool::DbPool, llm::client::LLMClient};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub llm: Arc<LLMClient>,
    pub config: Config,
}

impl AppState {
    pub fn new(db: DbPool, llm: LLMClient, config: Config) -> Self {
        Self {
            db,
            llm: Arc::new(llm),
            config,
        }
    }
}

