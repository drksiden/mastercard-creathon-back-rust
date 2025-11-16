use crate::{
    chat::session::MessageRole,
    error::AppError,
    state::AppState,
    utils::language::detect_language,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub session_id: String,  // Если не указан, будет создан новый
    #[serde(default)]
    pub user_id: String,  // Для идентификации пользователя
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub message: String,
    pub session_id: String,
    pub response_time_ms: u64,
}

pub async fn handle_chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let start = Instant::now();
    
    let session_id = if req.session_id.is_empty() {
        // Генерируем новый session_id
        uuid::Uuid::new_v4().to_string()
    } else {
        req.session_id.clone()
    };
    
    let user_id = if req.user_id.is_empty() {
        "anonymous".to_string()
    } else {
        req.user_id.clone()
    };
    
    tracing::info!("Chat request: session_id={}, user_id={}, message_len={}", 
        session_id, user_id, req.message.len());
    
    // Получаем или создаем сессию
    let mut session = state.sessions.get_or_create_session(session_id.clone(), user_id).await;
    
    // Добавляем сообщение пользователя
    session.add_message(MessageRole::User, req.message.clone());
    
    // Определяем язык
    let language = detect_language(&req.message);
    
    // Получаем контекст из последних сообщений (максимум 10)
    let recent_messages = session.get_recent_messages(10);
    
    // Генерируем ответ через LLM
    let response = state.llm.generate_chat_response(
        &req.message,
        &recent_messages.iter().map(|m| {
            (m.role.clone(), m.content.clone())
        }).collect::<Vec<_>>(),
        &language,
    ).await?;
    
    // Добавляем ответ ассистента
    session.add_message(MessageRole::Assistant, response.clone());
    
    // Сохраняем обновленную сессию
    state.sessions.update_session(session).await;
    
    let response_time = start.elapsed().as_millis() as u64;
    
    tracing::info!("Chat response generated: session_id={}, response_time={}ms", 
        session_id, response_time);
    
    Ok(Json(ChatResponse {
        message: response,
        session_id,
        response_time_ms: response_time,
    }))
}

