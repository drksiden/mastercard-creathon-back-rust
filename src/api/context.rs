use crate::{api::models::ClearContextRequest, error::AppError, state::AppState};
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ClearContextResponse {
    pub success: bool,
    pub message: String,
}

pub async fn handle_clear_context(
    State(state): State<AppState>,
    Json(req): Json<ClearContextRequest>,
) -> Result<Json<ClearContextResponse>, AppError> {
    let user_id = req.user_id.unwrap_or_else(|| "anonymous".to_string());
    
    state.query_context.clear_context(&user_id).await;
    
    tracing::info!("Cleared query context for user_id: {}", user_id);
    
    Ok(Json(ClearContextResponse {
        success: true,
        message: format!("Context cleared for user: {}", user_id),
    }))
}

