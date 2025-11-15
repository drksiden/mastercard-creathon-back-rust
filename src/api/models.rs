use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::analysis::AnalysisResult;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: String,
    #[serde(default)]
    pub include_analysis: bool,  // Whether to include LLM analysis
    #[serde(default)]
    pub use_cache: bool,  // Whether to use cache (default: true)
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub question: String,
    pub sql: String,
    pub data: Vec<serde_json::Value>,
    pub execution_time_ms: u64,
    pub row_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<AnalysisResult>,  // LLM analysis if requested
    #[serde(default)]
    pub cached: bool,  // Whether result was from cache
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub database: String,
    pub llm: String,
}

