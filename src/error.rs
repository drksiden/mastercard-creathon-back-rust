use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("LLM error: {0}")]
    LLM(#[from] anyhow::Error),
    
    #[error("Invalid SQL query: {0}")]
    InvalidSQL(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
            AppError::LLM(e) => {
                tracing::error!("LLM error: {:?}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            AppError::InvalidSQL(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Config(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

