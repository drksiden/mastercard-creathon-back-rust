mod health;
pub mod models;
mod query;
mod context;

use axum::{routing::{get, post}, Router};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/query", post(query::handle_query))
        .route("/chat", post(crate::chat::handler::handle_chat))
        .route("/context/clear", post(context::handle_clear_context))
}


