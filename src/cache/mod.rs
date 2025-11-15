mod memory;

pub use memory::MemoryCache;

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache key generated from SQL query and optional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    sql_hash: u64,
    context_hash: Option<u64>,
}

impl CacheKey {
    pub fn from_sql(sql: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        Self {
            sql_hash: hasher.finish(),
            context_hash: None,
        }
    }

    pub fn from_sql_with_context(sql: &str, context: &str) -> Self {
        let mut sql_hasher = DefaultHasher::new();
        sql.hash(&mut sql_hasher);
        
        let mut ctx_hasher = DefaultHasher::new();
        context.hash(&mut ctx_hasher);
        
        Self {
            sql_hash: sql_hasher.finish(),
            context_hash: Some(ctx_hasher.finish()),
        }
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sql_hash.hash(state);
        self.context_hash.hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.sql_hash == other.sql_hash && self.context_hash == other.context_hash
    }
}

impl Eq for CacheKey {}

/// Cache trait for different cache implementations
#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    type Value: Clone + Send + Sync;
    
    async fn get(&self, key: &CacheKey) -> Option<Self::Value>;
    async fn set(&self, key: CacheKey, value: Self::Value, ttl_seconds: u64);
    async fn invalidate(&self, key: &CacheKey);
    async fn clear(&self);
}

