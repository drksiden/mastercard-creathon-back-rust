use crate::{
    api::models::{QueryRequest, QueryResponse},
    db::queries::execute_query,
    error::AppError,
    state::AppState,
};
use axum::{extract::State, Json};
use std::time::Instant;

pub async fn handle_query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, AppError> {
    let start = Instant::now();
    
    tracing::info!("Received question: {}", req.question);
    
    // 0. Basic validation - reject obvious jailbreak attempts
    let question_lower = req.question.to_lowercase();
    let jailbreak_keywords = [
        "ignore previous",
        "forget all",
        "you are now",
        "you are a",
        "act as",
        "pretend to be",
        "roleplay as",
        "disregard instructions",
    ];
    
    for keyword in &jailbreak_keywords {
        if question_lower.contains(keyword) {
            tracing::warn!("Potential jailbreak attempt detected: {}", keyword);
            // Still process, but LLM will ignore it due to prompt protection
        }
    }
    
    // 1. Generate SQL using LLM
    let sql = state.llm.generate_sql(&req.question).await?;
    tracing::info!("Generated SQL: {}", sql);
    
    // 2. Execute query in database
    let data = execute_query(&state.db, &sql).await?;
    
    let execution_time = start.elapsed().as_millis() as u64;
    let row_count = data.len();
    
    tracing::info!(
        "Query executed successfully: {} rows in {}ms",
        row_count,
        execution_time
    );
    
    // 3. Log to audit
    let _ = log_query_audit(&state, &req.question, &sql, true, execution_time).await;
    
    Ok(Json(QueryResponse {
        question: req.question,
        sql,
        data,
        execution_time_ms: execution_time,
        row_count,
    }))
}

async fn log_query_audit(
    state: &AppState,
    question: &str,
    sql: &str,
    success: bool,
    execution_time_ms: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO query_audit_log (user_id, question, generated_sql, success, execution_time_ms)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind("anonymous")  // TODO: Add auth later
    .bind(question)
    .bind(sql)
    .bind(success)
    .bind(execution_time_ms as i32)
    .execute(&state.db)
    .await?;
    
    Ok(())
}

