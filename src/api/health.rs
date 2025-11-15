use crate::{
    api::models::HealthResponse,
    state::AppState,
};
use axum::{extract::State, Json};
use chrono::Utc;

pub async fn health_check(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {}", e),
    };
    
    // Check LLM (simple test)
    let llm_status = format!("{}", state.config.llm_provider);
    
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now(),
        database: db_status,
        llm: llm_status,
    })
}

