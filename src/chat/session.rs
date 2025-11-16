use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(session_id: String, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: session_id,
            user_id,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message {
            role,
            content,
            timestamp: Utc::now(),
        });
        self.updated_at = Utc::now();
    }

    pub fn get_recent_messages(&self, limit: usize) -> Vec<&Message> {
        let start = self.messages.len().saturating_sub(limit);
        self.messages[start..].iter().collect()
    }
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    max_session_age_hours: u64,
}

impl SessionManager {
    pub fn new(max_session_age_hours: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_session_age_hours,
        }
    }

    pub async fn get_or_create_session(&self, session_id: String, user_id: String) -> Session {
        let mut sessions = self.sessions.write().await;
        
        // Cleanup old sessions periodically
        self.cleanup_old_sessions(&mut sessions).await;
        
        sessions
            .entry(session_id.clone())
            .or_insert_with(|| Session::new(session_id, user_id))
            .clone()
    }

    pub async fn update_session(&self, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    async fn cleanup_old_sessions(&self, sessions: &mut HashMap<String, Session>) {
        let cutoff = Utc::now() - chrono::Duration::hours(self.max_session_age_hours as i64);
        sessions.retain(|_, session| session.updated_at > cutoff);
    }

    pub async fn clear_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }
}

