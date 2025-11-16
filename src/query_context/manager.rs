use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub question: String,
    pub sql: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserQueryContext {
    pub user_id: String,
    pub queries: Vec<QueryContext>,
    pub updated_at: DateTime<Utc>,
}

impl UserQueryContext {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            queries: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_query(&mut self, question: String, sql: String) {
        self.queries.push(QueryContext {
            question,
            sql,
            timestamp: Utc::now(),
        });
        
        // Храним только последние 10 запросов
        if self.queries.len() > 10 {
            self.queries.remove(0);
        }
        
        self.updated_at = Utc::now();
    }

    pub fn get_recent_queries(&self, limit: usize) -> Vec<&QueryContext> {
        let start = self.queries.len().saturating_sub(limit);
        self.queries[start..].iter().collect()
    }

    pub fn clear(&mut self) {
        self.queries.clear();
        self.updated_at = Utc::now();
    }
}

pub struct QueryContextManager {
    contexts: Arc<RwLock<HashMap<String, UserQueryContext>>>,
    max_context_age_hours: u64,
}

impl QueryContextManager {
    pub fn new(max_context_age_hours: u64) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            max_context_age_hours,
        }
    }

    pub async fn get_or_create_context(&self, user_id: String) -> UserQueryContext {
        let mut contexts = self.contexts.write().await;
        
        // Периодическая очистка старых контекстов
        self.cleanup_old_contexts(&mut contexts).await;
        
        contexts
            .entry(user_id.clone())
            .or_insert_with(|| UserQueryContext::new(user_id))
            .clone()
    }

    pub async fn update_context(&self, context: UserQueryContext) {
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.user_id.clone(), context);
    }

    pub async fn get_context(&self, user_id: &str) -> Option<UserQueryContext> {
        let contexts = self.contexts.read().await;
        contexts.get(user_id).cloned()
    }

    async fn cleanup_old_contexts(&self, contexts: &mut HashMap<String, UserQueryContext>) {
        let cutoff = Utc::now() - chrono::Duration::hours(self.max_context_age_hours as i64);
        contexts.retain(|_, context| context.updated_at > cutoff);
    }

    pub async fn clear_context(&self, user_id: &str) {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(user_id) {
            context.clear();
        }
    }

    pub async fn remove_context(&self, user_id: &str) {
        let mut contexts = self.contexts.write().await;
        contexts.remove(user_id);
    }
}

