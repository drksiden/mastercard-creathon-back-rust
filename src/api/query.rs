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
    
    // Определяем user_id (из запроса или генерируем анонимный)
    let user_id = req.user_id.clone().unwrap_or_else(|| {
        // Если не указан, используем session_id или генерируем анонимный
        req.session_id.clone().unwrap_or_else(|| "anonymous".to_string())
    });
    
    tracing::info!("Received question: {} (user_id: {}, analysis: {}, cache: {})", 
        req.question, user_id, req.include_analysis, req.use_cache);
    
    // 0. Проверка безопасности пользователя
    let (is_safe, safety_message) = state.user_safety.check_message_safety(&user_id, &req.question).await;
    if !is_safe {
        if let Some(ban_msg) = safety_message {
            return Err(AppError::BadRequest(ban_msg));
        }
    }
    
    // 0.1. Basic validation - reject obvious jailbreak attempts (дополнительная проверка)
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
            // Записываем нарушение
            if let Some(warning) = state.user_safety.record_violation(
                &user_id,
                crate::utils::user_safety::ViolationType::JailbreakAttempt,
                format!("Обнаружена попытка jailbreak: {}", keyword),
            ).await {
                tracing::warn!("User {} received warning: {}", user_id, warning);
            }
        }
    }
    
    // 0. Убираем префикс SQL из вопроса для обработки
    let question_clean = req.question.trim();
    let has_sql_prefix = question_clean.to_lowercase().starts_with("sql:");
    
    let question_clean = if question_clean.to_lowercase().starts_with("sql:") {
        question_clean.split_once(':').map(|(_, rest)| rest.trim()).unwrap_or(question_clean)
    } else {
        question_clean
    };
    
    // Определяем язык из очищенного вопроса (без префикса sql:)
    use crate::utils::language::detect_language;
    let language = detect_language(question_clean);
           
           // 1. Определяем, является ли вопрос запросом к БД или обычным вопросом
           // Если есть SQL префикс или это suggested question (обычно они SQL запросы), считаем SQL запросом
           use crate::utils::question_classifier::is_database_query;
           let is_db_query = has_sql_prefix || is_database_query(&req.question);
    
    if !is_db_query {
        // Это обычный вопрос, не про базу данных - отвечаем как в чате
        tracing::info!("Question classified as regular chat, not database query");
        
        // Генерируем обычный текстовый ответ
        let text_response = state.llm.generate_chat_response(
            question_clean,
            &[], // Нет истории для /api/query (можно добавить позже)
            &language,
        ).await?;
        
        let total_time = start.elapsed().as_millis() as u64;
        
        // Логируем как обычный вопрос
        let _ = log_query_audit(&state, &req.question, "", true, total_time).await;
        
        return Ok(Json(QueryResponse {
            question: req.question,
            sql: String::new(),
            text_response: Some(text_response),
            data: vec![],
            table: None,
            chart_data: None,
            execution_time_ms: total_time,
            row_count: 0,
            analysis: None,
            cached: false,
        }));
    }
    
    // 2. Это SQL-запрос - генерируем SQL с учетом контекста
    let context = state.query_context.get_or_create_context(user_id.clone()).await;
    let previous_queries: Vec<&crate::query_context::QueryContext> = 
        context.get_recent_queries(10).iter().map(|q| *q).collect();
    
    tracing::info!("Using context: {} previous queries", previous_queries.len());
    
    let sql = match state.llm.generate_sql(question_clean, &previous_queries).await {
        Ok(sql) => {
            tracing::info!("Generated SQL: {}", sql);
            sql
        }
        Err(e) => {
            tracing::error!("Failed to generate SQL: {}. Trying chat API as fallback...", e);
            // Если не удалось сгенерировать SQL, пробуем ответить через chat API
            let text_response = state.llm.generate_chat_response(
                question_clean,
                &[],
                &language,
            ).await?;
            
            let total_time = start.elapsed().as_millis() as u64;
            let _ = log_query_audit(&state, &req.question, "", false, total_time).await;
            
            return Ok(Json(QueryResponse {
                question: req.question,
                sql: String::new(),
                text_response: Some(text_response),
                data: vec![],
                table: None,
                chart_data: None,
                execution_time_ms: total_time,
                row_count: 0,
                analysis: None,
                cached: false,
            }));
        }
    };
    
    // 3. Это SQL-запрос - выполняем его
    // Check cache if enabled
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
            match execute_query(&state.db, &sql).await {
                Ok(result) => {
                    data = result;
                    execution_time = query_start.elapsed().as_millis() as u64;
                    row_count = data.len();
                }
                Err(e) => {
                    // Если ошибка SQL, возможно это обычный вопрос
                    let error_str = e.to_string();
                    if error_str.contains("syntax error") || 
                       error_str.contains("invalid input syntax") ||
                       error_str.contains("column") && error_str.contains("does not exist") {
                        tracing::warn!("SQL execution error, treating as regular question: {}", error_str);
                        
                        // Генерируем обычный текстовый ответ
                        let text_response = state.llm.generate_chat_response(
                            &req.question,
                            &[],
                            &language,
                        ).await?;
                        
                        let total_time = start.elapsed().as_millis() as u64;
                        let _ = log_query_audit(&state, &req.question, "", true, total_time).await;
                        
                        return Ok(Json(QueryResponse {
                            question: req.question,
                            sql: String::new(),
                            text_response: Some(text_response),
                            data: vec![],
                            table: None,
                            chart_data: None,
                            execution_time_ms: total_time,
                            row_count: 0,
                            analysis: None,
                            cached: false,
                        }));
                    }
                    // Для других ошибок пробрасываем дальше
                    return Err(e.into());
                }
            }
            
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
        match execute_query(&state.db, &sql).await {
            Ok(result) => {
                data = result;
                execution_time = query_start.elapsed().as_millis() as u64;
                row_count = data.len();
            }
            Err(e) => {
                // Если ошибка SQL, возможно это обычный вопрос
                let error_str = e.to_string();
                if error_str.contains("syntax error") || 
                   error_str.contains("invalid input syntax") ||
                   error_str.contains("column") && error_str.contains("does not exist") {
                    tracing::warn!("SQL execution error, treating as regular question: {}", error_str);
                    
                    // Генерируем обычный текстовый ответ
                    let text_response = state.llm.generate_chat_response(
                        &req.question,
                        &[],
                        &language,
                    ).await?;
                    
                    let total_time = start.elapsed().as_millis() as u64;
                    let _ = log_query_audit(&state, &req.question, "", true, total_time).await;
                    
                    return Ok(Json(QueryResponse {
                        question: req.question,
                        sql: String::new(),
                        text_response: Some(text_response),
                        data: vec![],
                        table: None,
                        chart_data: None,
                        execution_time_ms: total_time,
                        row_count: 0,
                        analysis: None,
                        cached: false,
                    }));
                }
                // Для других ошибок пробрасываем дальше
                return Err(e.into());
            }
        }
    }
    
    let total_time = start.elapsed().as_millis() as u64;
    
    tracing::info!(
        "Query executed successfully: {} rows in {}ms (cached: {})",
        row_count,
        execution_time,
        cached
    );
    
    // 4. Generate analysis (always for SQL queries to provide text description)
    // Analysis is needed to provide human-readable text descriptions instead of just tables
    let analysis = if req.include_analysis || is_db_query {
        tracing::info!("Generating LLM analysis for question: {}", req.question);
        match state.analysis.analyze_results(&req.question, &sql, &data, &language).await {
            Ok(analysis_result) => {
                tracing::info!("Analysis generated successfully: headline='{}', insights={}", 
                    analysis_result.headline, analysis_result.insights.len());
                Some(analysis_result)
            }
            Err(e) => {
                tracing::error!("Failed to generate analysis: {} (question: {}, sql: {})", 
                    e, req.question, sql);
                // Генерируем умный fallback анализ на основе данных
                Some(generate_fallback_analysis(&req.question, &data, row_count, &language))
            }
        }
    } else {
        None
    };
    
    // 5. Сохраняем запрос в контекст пользователя
    let mut updated_context = state.query_context.get_or_create_context(user_id.clone()).await;
    updated_context.add_query(req.question.clone(), sql.clone());
    state.query_context.update_context(updated_context).await;
    
    // 6. Log to audit
    let _ = log_query_audit(&state, &req.question, &sql, true, total_time).await;
    
    // 7. Форматируем данные в зависимости от output_type
    use crate::utils::formatters;
    use crate::api::models::OutputType;
    
    // Генерируем таблицу только если она действительно нужна
    // Не генерируем таблицу для:
    // - JSON вывода
    // - Одиночных значений (COUNT, SUM, AVG без GROUP BY)
    // - Когда есть анализ и он рекомендует не показывать таблицу
    let table = match &req.output_type {
        OutputType::Json => None,  // Только для JSON не генерируем таблицу
        _ => {
            // Определяем, нужна ли таблица
            // Таблица НЕ нужна для:
            // - Одиночных агрегированных значений (COUNT, SUM, AVG без GROUP BY)
            // - Когда пользователь явно не просил таблицу
            let needs_table = if data.is_empty() {
                false
            } else if data.len() == 1 {
                // Для одного значения проверяем, это агрегация или нет
                if let Some(first_obj) = data[0].as_object() {
                    let keys: Vec<&String> = first_obj.keys().collect();
                    // Если только одно поле - скорее всего агрегация, таблица не нужна
                    if keys.len() == 1 {
                        let key = keys[0];
                        let key_lower = key.to_lowercase();
                        // Если это агрегация (count, sum, avg, total, max, min, etc.) - таблица не нужна
                        // Таблица нужна только если это категориальные данные (city, category, type, etc.)
                        if key_lower.contains("count") || key_lower.contains("sum") || 
                           key_lower.contains("avg") || key_lower.contains("total") ||
                           key_lower.contains("average") || key_lower.contains("amount") ||
                           key_lower.contains("max") || key_lower.contains("min") ||
                           key_lower.contains("maximum") || key_lower.contains("minimum") {
                            false  // Агрегация - таблица не нужна, только текстовое описание
                        } else {
                            // Проверяем, не просил ли пользователь таблицу явно
                            let question_lower = req.question.to_lowercase();
                            question_lower.contains("таблица") || question_lower.contains("table") || 
                            question_lower.contains("таблицу") || question_lower.contains("csv")
                        }
                    } else {
                        // Несколько полей - может быть таблица, но только если не все агрегации
                        let all_aggregations = keys.iter().all(|k| {
                            let k_lower = k.to_lowercase();
                            k_lower.contains("count") || k_lower.contains("sum") || 
                            k_lower.contains("avg") || k_lower.contains("total") ||
                            k_lower.contains("average") || k_lower.contains("amount") ||
                            k_lower.contains("max") || k_lower.contains("min")
                        });
                        if all_aggregations {
                            // Все поля - агрегации, таблица не нужна
                            let question_lower = req.question.to_lowercase();
                            question_lower.contains("таблица") || question_lower.contains("table") || 
                            question_lower.contains("таблицу") || question_lower.contains("csv")
                        } else {
                            true  // Есть категориальные поля - таблица может быть полезна
                        }
                    }
                } else {
                    true
                }
            } else {
                // Для нескольких строк - таблица нужна только если пользователь явно просил
                let question_lower = req.question.to_lowercase();
                question_lower.contains("таблица") || question_lower.contains("table") || 
                question_lower.contains("таблицу") || question_lower.contains("csv") ||
                question_lower.contains("показать") || question_lower.contains("вывести") ||
                question_lower.contains("список") || question_lower.contains("list")
            };
            
            if needs_table && !data.is_empty() {
                Some(formatters::format_as_table(&data))
            } else {
                None
            }
        }
    };
    
    let chart_data = match &req.output_type {
        OutputType::Chart => {
            let chart_type = if let Some(analysis) = &analysis {
                analysis.chart_type.as_ref()
                    .map(|ct| format!("{:?}", ct).to_lowercase())
                    .unwrap_or_else(|| "auto".to_string())
            } else {
                "auto".to_string()
            };
            formatters::format_as_chart_data(&data, &chart_type)
        }
        OutputType::Auto => {
            // Автоматически определяем, нужна ли диаграмма
            // Для запросов с "график", "нарисуй" и т.д. всегда генерируем chart_data
            let question_lower = req.question.to_lowercase();
            let should_generate_chart = question_lower.contains("график") || question_lower.contains("диаграмма") ||
                                       question_lower.contains("chart") || question_lower.contains("graph") ||
                                       question_lower.contains("нарисуй") || question_lower.contains("построй") ||
                                       question_lower.contains("визуализац") || question_lower.contains("visualization");
            
            if should_generate_chart || (row_count > 1 && row_count <= 20) {
                let chart_type = if let Some(analysis) = &analysis {
                    analysis.chart_type.as_ref()
                        .map(|ct| format!("{:?}", ct).to_lowercase())
                        .unwrap_or_else(|| "auto".to_string())
                } else {
                    "auto".to_string()
                };
                formatters::format_as_chart_data(&data, &chart_type)
            } else {
                None
            }
        }
        _ => None,
    };
    
    // 8. Prepare response (optionally hide SQL)
    let response_sql = if req.include_sql {
        sql.clone()
    } else {
        String::new()
    };
    
    Ok(Json(QueryResponse {
        question: req.question,
        sql: response_sql,
        text_response: None,  // SQL-запрос, текстового ответа нет
        data,
        table,
        chart_data,
        execution_time_ms: total_time,
        row_count,
        analysis,
        cached,
    }))
}

fn generate_fallback_analysis(
    _question: &str,
    data: &[serde_json::Value],
    row_count: usize,
    language: &crate::utils::language::Language,
) -> crate::analysis::AnalysisResult {
    use crate::analysis::{AnalysisResult, Insight, InsightSignificance, ChartType};
    use crate::utils::language::Language;
    
    let (found_records, result_contains, show_details, compare_periods, main_result, found_for, shows_records, query_result, all_categories, compare_others) = match language {
        Language::Russian => (
            "Найдено {} записей",
            "Результат запроса содержит {} записей",
            "Показать детализацию",
            "Сравнить с другими периодами",
            "Основной результат",
            "Найдено {} записей для {}",
            "Результат показывает {} записей для категории '{}'",
            "Результат запроса: {} строк данных",
            "Показать все категории",
            "Сравнить с другими",
        ),
        Language::English => (
            "Found {} records",
            "Query result contains {} records",
            "Show details",
            "Compare with other periods",
            "Main result",
            "Found {} records for {}",
            "Result shows {} records for category '{}'",
            "Query result: {} rows of data",
            "Show all categories",
            "Compare with others",
        ),
        Language::Kazakh => (
            "{} жазба табылды",
            "Сұрау нәтижесі {} жазбаны қамтиды",
            "Толық мәліметтерді көрсету",
            "Басқа кезеңдермен салыстыру",
            "Негізгі нәтиже",
            "{} үшін {} жазба табылды",
            "Нәтиже '{}' категориясы үшін {} жазбаны көрсетеді",
            "Сұрау нәтижесі: {} жол деректер",
            "Барлық категорияларды көрсету",
            "Басқалармен салыстыру",
        ),
    };
    
    // Пытаемся извлечь информацию из данных для headline
    if let Some(first_row) = data.first() {
        if let Some(obj) = first_row.as_object() {
            // Пытаемся найти ключевые поля
            if let Some(count) = obj.get("count").or_else(|| obj.get("transaction_count")).or_else(|| obj.get("total_transactions")) {
                if let Some(count_num) = count.as_u64().or_else(|| count.as_i64().map(|v| v as u64)) {
                    return AnalysisResult {
                        headline: found_records.replace("{}", &count_num.to_string()),
                        insights: vec![],
                        explanation: result_contains.replace("{}", &count_num.to_string()),
                        suggested_questions: vec![
                            show_details.to_string(),
                            compare_periods.to_string(),
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
                let category = obj.get(category_key).and_then(|v| v.as_str()).unwrap_or(match language {
                    Language::Russian => "категория",
                    Language::English => "category",
                    Language::Kazakh => "категория",
                });
                let count = obj.get(count_key).and_then(|v| v.as_u64()).unwrap_or(0);
                return AnalysisResult {
                    headline: found_records.replace("{}", &count.to_string()),
                    insights: vec![
                        Insight {
                            title: main_result.to_string(),
                            description: found_for.replace("{}", &count.to_string()).replace("{}", category),
                            significance: InsightSignificance::Medium,
                        }
                    ],
                    explanation: shows_records.replace("{}", &count.to_string()).replace("{}", category),
                    suggested_questions: vec![
                        all_categories.to_string(),
                        compare_others.to_string(),
                    ],
                    chart_type: if row_count > 1 && row_count <= 10 { Some(ChartType::Bar) } else { None },
                    data: data.to_vec(),
                };
            }
        }
    }
    
    // Базовый fallback
    AnalysisResult {
        headline: found_records.replace("{}", &row_count.to_string()),
        insights: vec![],
        explanation: query_result.replace("{}", &row_count.to_string()),
        suggested_questions: vec![
            show_details.to_string(),
            compare_periods.to_string(),
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

