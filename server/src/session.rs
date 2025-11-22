use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, Instant>>>,
    ttl: Duration,
}

impl SessionStore {
    pub fn new(ttl_minutes: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_minutes.saturating_mul(60)),
        }
    }

    pub async fn create_session(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let expires_at = Instant::now() + self.ttl;
        let mut guard = self.inner.write().await;
        guard.insert(id.clone(), expires_at);
        id
    }

    pub async fn validate(&self, id: &str) -> bool {
        self.prune_expired().await;
        let guard = self.inner.read().await;
        guard.get(id).is_some()
    }

    pub async fn remove(&self, id: &str) {
        let mut guard = self.inner.write().await;
        guard.remove(id);
    }

    async fn prune_expired(&self) {
        let mut guard = self.inner.write().await;
        let now = Instant::now();
        guard.retain(|_, &mut expires_at| expires_at > now);
    }
}
