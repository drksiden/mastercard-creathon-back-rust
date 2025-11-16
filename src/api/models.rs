use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::analysis::AnalysisResult;

#[derive(Debug, Deserialize, Clone)]
pub enum OutputType {
    #[serde(rename = "table")]
    Table,
    #[serde(rename = "chart")]
    Chart,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "auto")]
    Auto,  // Автоматически определяет на основе данных
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::Auto
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: String,
    #[serde(default)]
    pub include_analysis: bool,  // Whether to include LLM analysis
    #[serde(default)]
    pub use_cache: bool,  // Whether to use cache (default: true)
    #[serde(default)]
    pub include_sql: bool,  // Whether to include SQL in response (default: true for debugging)
    #[serde(default)]
    pub user_id: Option<String>,  // User ID for context (optional)
    #[serde(default)]
    pub session_id: Option<String>,  // Session ID (optional, for compatibility)
    #[serde(default)]
    pub output_type: OutputType,  // Тип вывода: table, chart, json, auto
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub question: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub sql: String,  // SQL скрывается если include_sql=false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_response: Option<String>,  // Текстовый ответ для обычных вопросов (не про БД)
    pub data: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,  // Форматированная таблица (markdown/csv)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,  // Данные для построения диаграммы
    pub execution_time_ms: u64,
    pub row_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<AnalysisResult>,  // LLM analysis if requested
    #[serde(default)]
    pub cached: bool,  // Whether result was from cache
}

#[derive(Debug, Serialize)]
pub struct ChartData {
    pub chart_type: String,  // bar, line, pie, etc.
    pub labels: Vec<String>,  // Метки для осей
    pub datasets: Vec<ChartDataset>,  // Данные для графиков
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub database: String,
    pub llm: String,
}

#[derive(Debug, Deserialize)]
pub struct ClearContextRequest {
    #[serde(default)]
    pub user_id: Option<String>,  // User ID для очистки контекста
}

