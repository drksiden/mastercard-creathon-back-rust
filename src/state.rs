use crate::{
    analysis::AnalysisClient,
    cache::MemoryCache,
    chat::session::SessionManager,
    config::Config,
    db::pool::DbPool,
    llm::client::LLMClient,
    query_context::QueryContextManager,
    utils::user_safety::UserSafetyManager,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub llm: Arc<LLMClient>,
    pub analysis: Arc<AnalysisClient>,
    pub cache: Arc<MemoryCache<CachedQueryResult>>,
    pub sessions: Arc<SessionManager>,
    pub query_context: Arc<QueryContextManager>,
    pub user_safety: Arc<UserSafetyManager>,
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
        let sessions = Arc::new(SessionManager::new(24)); // Сессии хранятся 24 часа
        let query_context = Arc::new(QueryContextManager::new(24)); // Контекст хранится 24 часа
        let user_safety = Arc::new(UserSafetyManager::new(5, 24)); // Макс 5 предупреждений, бан на 24 часа
        
        Self {
            db,
            llm: Arc::new(llm),
            analysis,
            cache,
            sessions,
            query_context,
            user_safety,
            config,
        }
    }
}

