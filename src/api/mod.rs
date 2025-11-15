mod health;
mod models;
mod query;

use axum::{routing::{get, post}, Router};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/query", post(query::handle_query))
}

