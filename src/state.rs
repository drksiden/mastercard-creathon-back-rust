use crate::{
    analysis::AnalysisClient,
    cache::MemoryCache,
    config::Config,
    db::pool::DbPool,
    llm::client::LLMClient,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub llm: Arc<LLMClient>,
    pub analysis: Arc<AnalysisClient>,
    pub cache: Arc<MemoryCache<CachedQueryResult>>,
    pub config: Config,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CachedQueryResult {
    pub sql: String,
    pub data: Vec<serde_json::Value>,
    pub execution_time_ms: u64,
    pub row_count: usize,
}

impl AppState {
    pub fn new(db: DbPool, llm: LLMClient, config: Config) -> Self {
        let analysis = Arc::new(AnalysisClient::new(config.clone()));
        let cache = Arc::new(MemoryCache::new());
        
        Self {
            db,
            llm: Arc::new(llm),
            analysis,
            cache,
            config,
        }
    }
}

