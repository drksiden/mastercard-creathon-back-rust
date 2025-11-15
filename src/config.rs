use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub llm_provider: String,  // "ollama" | "openai" | "gemini"
    pub ollama_url: String,
    pub ollama_model: String,
    pub openai_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub gemini_model: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Try to load .env file, but don't fail if it doesn't exist
        dotenvy::dotenv().ok();
        
        // DATABASE_URL может быть в формате postgresql+psycopg2:// (Python) или postgresql:// (Rust)
        // Преобразуем в формат для Rust
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!(
                "DATABASE_URL environment variable is required. \
                Please set it in .env file or export it: \
                export DATABASE_URL=postgresql://user:password@host:5432/dbname"
            ))?
            .replace("postgresql+psycopg2://", "postgresql://"); // Убираем +psycopg2 для Python
        
        Ok(Self {
            database_url,
            llm_provider: std::env::var("LLM_PROVIDER")
                .unwrap_or_else(|_| "ollama".to_string()),
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "mixtral:8x7b-instruct".to_string()),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            gemini_api_key: std::env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| std::env::var("LLM_API_KEY").ok()),
            gemini_model: std::env::var("GEMINI_MODEL")
                .unwrap_or_else(|_| "gemini-2.0-flash-exp".to_string()),
            host: std::env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        })
    }
}

