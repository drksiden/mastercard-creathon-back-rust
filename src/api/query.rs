use crate::{
    api::models::{QueryRequest, QueryResponse},
    cache::{Cache, CacheKey},
    db::queries::execute_query,
    error::AppError,
    state::{AppState, CachedQueryResult},
};
use axum::{extract::State, Json};
use std::time::Instant;

pub async fn handle_query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, AppError> {
    let start = Instant::now();
    
    tracing::info!("Received question: {} (analysis: {}, cache: {})", 
        req.question, req.include_analysis, req.use_cache);
    
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
    
    // 2. Check cache if enabled
    let cache_key = CacheKey::from_sql(&sql);
    let mut cached = false;
    let data;
    let execution_time;
    let row_count;
    
    if req.use_cache {
        if let Some(cached_result) = state.cache.get(&cache_key).await {
            tracing::info!("Cache hit for SQL query");
            cached = true;
            data = cached_result.data;
            execution_time = cached_result.execution_time_ms;
            row_count = cached_result.row_count;
        } else {
            // Cache miss - execute query
            let query_start = Instant::now();
            data = execute_query(&state.db, &sql).await?;
            execution_time = query_start.elapsed().as_millis() as u64;
            row_count = data.len();
            
            // Cache the result (TTL: 5 minutes for operational, 30 minutes for historical)
            let ttl = if sql.to_lowercase().contains("current_date") || 
                       sql.to_lowercase().contains("today") || 
                       sql.to_lowercase().contains("last") {
                300 // 5 minutes for time-sensitive queries
            } else {
                1800 // 30 minutes for historical data
            };
            
            let cached_result = CachedQueryResult {
                sql: sql.clone(),
                data: data.clone(),
                execution_time_ms: execution_time,
                row_count,
            };
            state.cache.set(cache_key, cached_result, ttl).await;
            tracing::info!("Cached query result (TTL: {}s)", ttl);
        }
    } else {
        // Cache disabled - execute query
        let query_start = Instant::now();
        data = execute_query(&state.db, &sql).await?;
        execution_time = query_start.elapsed().as_millis() as u64;
        row_count = data.len();
    }
    
    let total_time = start.elapsed().as_millis() as u64;
    
    tracing::info!(
        "Query executed successfully: {} rows in {}ms (cached: {})",
        row_count,
        execution_time,
        cached
    );
    
    // 3. Generate analysis if requested
    let analysis = if req.include_analysis {
        tracing::info!("Generating LLM analysis for question: {}", req.question);
        match state.analysis.analyze_results(&req.question, &sql, &data).await {
            Ok(analysis_result) => {
                tracing::info!("Analysis generated successfully: headline='{}', insights={}", 
                    analysis_result.headline, analysis_result.insights.len());
                Some(analysis_result)
            }
            Err(e) => {
                tracing::error!("Failed to generate analysis: {} (question: {}, sql: {})", 
                    e, req.question, sql);
                // Генерируем умный fallback анализ на основе данных
                Some(generate_fallback_analysis(&req.question, &data, row_count))
            }
        }
    } else {
        None
    };
    
    // 4. Log to audit
    let _ = log_query_audit(&state, &req.question, &sql, true, total_time).await;
    
    // 5. Prepare response (optionally hide SQL)
    // По умолчанию include_sql=true (для отладки), но можно скрыть
    let response_sql = if req.include_sql {
        sql.clone()
    } else {
        String::new()  // Пустая строка, поле будет пропущено в JSON
    };
    
    Ok(Json(QueryResponse {
        question: req.question,
        sql: response_sql,
        data,
        execution_time_ms: total_time,
        row_count,
        analysis,
        cached,
    }))
}

fn generate_fallback_analysis(
    question: &str,
    data: &[serde_json::Value],
    row_count: usize,
) -> crate::analysis::AnalysisResult {
    use crate::analysis::{AnalysisResult, Insight, InsightSignificance, ChartType};
    
    // Пытаемся извлечь информацию из данных для headline
    if let Some(first_row) = data.first() {
        if let Some(obj) = first_row.as_object() {
            // Пытаемся найти ключевые поля
            if let Some(count) = obj.get("count").or_else(|| obj.get("transaction_count")).or_else(|| obj.get("total_transactions")) {
                if let Some(count_num) = count.as_u64().or_else(|| count.as_i64().map(|v| v as u64)) {
                    return AnalysisResult {
                        headline: format!("Найдено {} записей", count_num),
                        insights: vec![],
                        explanation: format!("Результат запроса содержит {} записей", count_num),
                        suggested_questions: vec![
                            "Показать детализацию".to_string(),
                            "Сравнить с другими периодами".to_string(),
                        ],
                        chart_type: if row_count > 1 && row_count <= 10 { Some(ChartType::Bar) } else { None },
                        data: data.to_vec(),
                    };
                }
            }
            
            // Если есть категория и количество
            if let (Some(category_key), Some(count_key)) = (
                obj.keys().find(|k| k.contains("category") || k.contains("type") || k.contains("name")),
                obj.keys().find(|k| k.contains("count") || k.contains("total"))
            ) {
                let category = obj.get(category_key).and_then(|v| v.as_str()).unwrap_or("категория");
                let count = obj.get(count_key).and_then(|v| v.as_u64()).unwrap_or(0);
                return AnalysisResult {
                    headline: format!("{}: {} записей", category, count),
                    insights: vec![
                        Insight {
                            title: "Основной результат".to_string(),
                            description: format!("Найдено {} записей для {}", count, category),
                            significance: InsightSignificance::Medium,
                        }
                    ],
                    explanation: format!("Результат показывает {} записей для категории '{}'", count, category),
                    suggested_questions: vec![
                        "Показать все категории".to_string(),
                        "Сравнить с другими".to_string(),
                    ],
                    chart_type: if row_count > 1 && row_count <= 10 { Some(ChartType::Bar) } else { None },
                    data: data.to_vec(),
                };
            }
        }
    }
    
    // Базовый fallback
    AnalysisResult {
        headline: format!("Найдено {} записей", row_count),
        insights: vec![],
        explanation: format!("Результат запроса: {} строк данных", row_count),
        suggested_questions: vec![
            "Показать детализацию".to_string(),
            "Сравнить с другими периодами".to_string(),
        ],
        chart_type: if row_count > 1 && row_count <= 10 { Some(ChartType::Table) } else { None },
        data: data.to_vec(),
    }
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

