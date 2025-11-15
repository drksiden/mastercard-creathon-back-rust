use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: String,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub question: String,
    pub sql: String,
    pub data: Vec<serde_json::Value>,
    pub execution_time_ms: u64,
    pub row_count: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub database: String,
    pub llm: String,
}

