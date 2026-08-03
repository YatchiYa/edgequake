//! Global provider/model capacity pool (protects LLM/API backends).
//!
//! ## First principles
//!
//! Tenant fair-share prevents noisy neighbors. Provider pools prevent two
//! tenants from each holding `MAX=1` and double-hitting the same Ollama
//! (or saturating a shared API). Cloud providers default to unlimited
//! unless `EDGEQUAKE_PROVIDER_MAX_INFLIGHT` is set.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tracing::debug;

use crate::capacity_block::CapacityLayer;

/// Env override for global provider/model in-flight max (0 = unlimited).
pub const PROVIDER_MAX_INFLIGHT_ENV: &str = "EDGEQUAKE_PROVIDER_MAX_INFLIGHT";

/// Default max for local extract providers when env unset.
pub const LOCAL_PROVIDER_MAX_INFLIGHT_DEFAULT: usize = 1;

/// True for capacity-bound local LLM backends.
pub fn is_local_capacity_provider(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "ollama" | "lmstudio" | "lm-studio" | "lm_studio"
    )
}

/// Build the global pool key: local → `provider/model`, cloud → `provider`.
pub fn provider_pool_key(provider: &str, model: Option<&str>) -> String {
    let p = provider.trim().to_ascii_lowercase();
    if p.is_empty() {
        return "unknown".into();
    }
    if is_local_capacity_provider(&p) {
        let m = model.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("default");
        format!("{p}/{m}")
    } else {
        p
    }
}

/// Resolve max in-flight for a provider/model pool.
///
/// - `EDGEQUAKE_PROVIDER_MAX_INFLIGHT` set → that value (0 = unlimited).
/// - Local providers → [`LOCAL_PROVIDER_MAX_INFLIGHT_DEFAULT`].
/// - Cloud → `0` (unlimited / pool disabled).
pub fn resolve_provider_pool_max(provider: &str, _model: Option<&str>) -> usize {
    if let Ok(raw) = std::env::var(PROVIDER_MAX_INFLIGHT_ENV) {
        if let Ok(n) = raw.trim().parse::<usize>() {
            return n;
        }
    }
    if is_local_capacity_provider(provider) {
        LOCAL_PROVIDER_MAX_INFLIGHT_DEFAULT
    } else {
        0
    }
}

/// Pure resolver for tests (no env).
pub fn resolve_provider_pool_max_from(
    provider: &str,
    env_override: Option<usize>,
) -> usize {
    if let Some(n) = env_override {
        return n;
    }
    if is_local_capacity_provider(provider) {
        LOCAL_PROVIDER_MAX_INFLIGHT_DEFAULT
    } else {
        0
    }
}

/// Global process-local provider/model semaphore map.
#[derive(Clone, Debug)]
pub struct ProviderCapacityLimiter {
    /// 0 = unlimited (try_acquire always succeeds without a permit).
    max: usize,
    provider: String,
    model: Option<String>,
    pool_key: String,
    semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
}

impl ProviderCapacityLimiter {
    /// Create a limiter for one provider/model identity.
    pub fn new(provider: impl Into<String>, model: Option<String>, max: usize) -> Self {
        let provider = provider.into();
        let pool_key = provider_pool_key(&provider, model.as_deref());
        Self {
            max,
            provider,
            model,
            pool_key,
            semaphores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// From env + provider/model (boot wiring).
    pub fn from_env(provider: impl Into<String>, model: Option<String>) -> Self {
        let provider = provider.into();
        let max = resolve_provider_pool_max(&provider, model.as_deref());
        Self::new(provider, model, max)
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn pool_key(&self) -> &str {
        &self.pool_key
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    pub fn is_unlimited(&self) -> bool {
        self.max == 0
    }

    async fn semaphore(&self) -> Option<Arc<Semaphore>> {
        if self.max == 0 {
            return None;
        }
        let read = self.semaphores.read().await;
        if let Some(s) = read.get(&self.pool_key) {
            return Some(Arc::clone(s));
        }
        drop(read);
        let mut write = self.semaphores.write().await;
        let sem = write.entry(self.pool_key.clone()).or_insert_with(|| {
            debug!(
                pool_key = %self.pool_key,
                max = self.max,
                "Created provider capacity semaphore"
            );
            Arc::new(Semaphore::new(self.max))
        });
        Some(Arc::clone(sem))
    }

    pub async fn active_count(&self) -> usize {
        let Some(sem) = self.semaphore().await else {
            return 0;
        };
        self.max.saturating_sub(sem.available_permits())
    }

    /// Try acquire; `None` means blocked. Unlimited → `Some(None)` (no permit).
    pub async fn try_acquire(&self) -> Result<Option<OwnedSemaphorePermit>, CapacityLayer> {
        let Some(sem) = self.semaphore().await else {
            return Ok(None);
        };
        match sem.try_acquire_owned() {
            Ok(p) => Ok(Some(p)),
            Err(_) => Err(CapacityLayer::ProviderModel {
                provider: self.provider.clone(),
                model: self.model.clone(),
                in_use: self.max,
                max: self.max,
            }),
        }
    }

    /// Block until a provider slot is free (unlimited → dummy no-op via early return).
    pub async fn acquire(&self) -> Result<Option<OwnedSemaphorePermit>, tokio::sync::AcquireError> {
        let Some(sem) = self.semaphore().await else {
            return Ok(None);
        };
        Ok(Some(sem.acquire_owned().await?))
    }

    pub fn blocked_layer(&self, in_use: usize) -> CapacityLayer {
        CapacityLayer::ProviderModel {
            provider: self.provider.clone(),
            model: self.model.clone(),
            in_use,
            max: self.max.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_key_includes_model() {
        assert_eq!(
            provider_pool_key("ollama", Some("gemma3:latest")),
            "ollama/gemma3:latest"
        );
    }

    #[test]
    fn cloud_key_is_provider_only() {
        assert_eq!(provider_pool_key("mistral", Some("mistral-small")), "mistral");
    }

    #[test]
    fn resolve_local_default_is_one() {
        assert_eq!(resolve_provider_pool_max_from("ollama", None), 1);
        assert_eq!(resolve_provider_pool_max_from("openai", None), 0);
    }

    #[tokio::test]
    async fn provider_max_one_blocks_second() {
        let lim = ProviderCapacityLimiter::new("ollama", Some("gemma3".into()), 1);
        let p1 = lim.try_acquire().await.expect("first ok").expect("permit");
        let blocked = lim.try_acquire().await.expect_err("second blocked");
        assert!(matches!(blocked, CapacityLayer::ProviderModel { max: 1, .. }));
        drop(p1);
        let _p2 = lim.try_acquire().await.expect("after drop").expect("permit");
    }

    #[tokio::test]
    async fn unlimited_never_blocks() {
        let lim = ProviderCapacityLimiter::new("mistral", None, 0);
        assert!(lim.is_unlimited());
        assert!(lim.try_acquire().await.unwrap().is_none());
        assert!(lim.try_acquire().await.unwrap().is_none());
    }
}
