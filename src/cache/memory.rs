use super::{Cache, CacheKey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// In-memory cache implementation with TTL support
pub struct MemoryCache<T> {
    data: Arc<RwLock<HashMap<CacheKey, CacheEntry<T>>>>,
}

impl<T> MemoryCache<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let now = Instant::now();
        data.retain(|_, entry| entry.expires_at > now);
    }
}

#[async_trait::async_trait]
impl<T> Cache for MemoryCache<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Value = T;

    async fn get(&self, key: &CacheKey) -> Option<Self::Value> {
        // Cleanup expired entries periodically (every 10th request)
        if rand::random::<u8>() % 10 == 0 {
            self.cleanup_expired().await;
        }

        let data = self.data.read().await;
        let entry = data.get(key)?;
        
        if entry.expires_at <= Instant::now() {
            // Entry expired, but we'll let cleanup handle it
            return None;
        }
        
        Some(entry.value.clone())
    }

    async fn set(&self, key: CacheKey, value: Self::Value, ttl_seconds: u64) {
        let expires_at = Instant::now() + Duration::from_secs(ttl_seconds);
        let entry = CacheEntry { value, expires_at };
        
        let mut data = self.data.write().await;
        data.insert(key, entry);
    }

    async fn invalidate(&self, key: &CacheKey) {
        let mut data = self.data.write().await;
        data.remove(key);
    }

    async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }
}

impl<T> Default for MemoryCache<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

