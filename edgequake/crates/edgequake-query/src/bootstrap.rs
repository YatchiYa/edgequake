//! Production query-engine bootstrap (SPEC-022 P-H4 / DRY SSOT).
//!
//! Shared by `edgequake-api` HTTP bootstrap and `edgequake-core` orchestrator so
//! SDK and API query quality stay identical. Reranker resolution delegates to
//! `edgequake-llm` factory (SPEC-024 2.4), with optional DashScope intl/model
//! overrides for Acc CE ablations (SPEC-001).

use std::sync::Arc;

use edgequake_llm::traits::{EmbeddingProvider, LLMProvider};
use edgequake_llm::{HttpReranker, RerankConfig, Reranker};
use tracing::info;

use edgequake_storage::traits::{GraphStorage, KVStorage, VectorStorage};

use crate::{QueryEngine, QueryEngineConfig};

/// Default DashScope intl text-rerank endpoint (qwen3-rerank).
///
/// Appends `?compat=dashscope.aliyuncs.com` so `HttpReranker::new` keeps the
/// Aliyun nested request/response format (`detect_format` matches that substring).
const DEFAULT_DASHSCOPE_INTL_RERANK_URL: &str = "https://dashscope-intl.aliyuncs.com/api/v1/services/rerank/text-rerank/text-rerank?compat=dashscope.aliyuncs.com";

const DEFAULT_DASHSCOPE_RERANK_MODEL: &str = "qwen3-rerank";

/// Create the configured reranker for production (BM25 by default).
///
/// Set `EDGEQUAKE_RERANKER=cross_encoder` for neural reranking — see
/// [`edgequake_llm::create_production_reranker`].
///
/// Acc / intl overrides (when `cross_encoder`):
/// - `EDGEQUAKE_RERANKER_BASE_URL` — default DashScope intl text-rerank URL
/// - `EDGEQUAKE_RERANKER_MODEL` — default `qwen3-rerank`
/// - `DASHSCOPE_API_KEY` / `ALIYUN_API_KEY`
pub fn create_production_reranker() -> Arc<dyn Reranker> {
    create_production_reranker_with_embedding(None)
}

/// Same as [`create_production_reranker`] but passes embedding provider for bi-encoder fallback.
pub fn create_production_reranker_with_embedding(
    embedding: Option<Arc<dyn EmbeddingProvider>>,
) -> Arc<dyn Reranker> {
    if let Some(reranker) = try_dashscope_override_reranker() {
        return reranker;
    }
    edgequake_llm::create_production_reranker(embedding)
}

/// Prefer explicit DashScope intl/model overrides over china `gte-rerank-v2` defaults.
fn try_dashscope_override_reranker() -> Option<Arc<dyn Reranker>> {
    let mode = std::env::var("EDGEQUAKE_RERANKER")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if mode != "cross_encoder" {
        return None;
    }

    let provider = std::env::var("EDGEQUAKE_RERANKER_PROVIDER")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let want_dashscope = matches!(provider.as_str(), "" | "aliyun" | "dashscope");
    if !want_dashscope {
        return None;
    }

    let api_key = env_api_key("DASHSCOPE_API_KEY").or_else(|| env_api_key("ALIYUN_API_KEY"))?;

    let base_url = std::env::var("EDGEQUAKE_RERANKER_BASE_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_DASHSCOPE_INTL_RERANK_URL.to_string());

    let model = std::env::var("EDGEQUAKE_RERANKER_MODEL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_DASHSCOPE_RERANK_MODEL.to_string());

    // Force Aliyun nested format even when URL host is dashscope-intl.
    let mut url = base_url;
    if url.contains("dashscope") && !url.contains("dashscope.aliyuncs.com") {
        let sep = if url.contains('?') { '&' } else { '?' };
        url = format!("{url}{sep}compat=dashscope.aliyuncs.com");
    }

    info!(
        base_url = %url,
        model = %model,
        "Using DashScope HTTP cross-encoder reranker (SPEC-001 override)"
    );

    let config = RerankConfig {
        model,
        base_url: url,
        api_key: Some(api_key),
        ..RerankConfig::aliyun("unused")
    };
    Some(Arc::new(HttpReranker::new(config)) as Arc<dyn Reranker>)
}

fn env_api_key(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && !v.to_ascii_uppercase().starts_with("FAKE"))
}

/// Build the production query engine: reranker + embedding cache + result cache.
pub fn build_production_query_engine(
    vector_storage: Arc<dyn VectorStorage>,
    graph_storage: Arc<dyn GraphStorage>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    llm_provider: Arc<dyn LLMProvider>,
    kv_storage: Option<Arc<dyn KVStorage>>,
) -> Arc<QueryEngine> {
    let reranker = create_production_reranker_with_embedding(Some(Arc::clone(&embedding_provider)));
    let mut engine = QueryEngine::new(
        QueryEngineConfig::default(),
        vector_storage,
        graph_storage,
        embedding_provider,
        llm_provider,
    )
    .with_reranker(reranker)
    .with_embedding_cache()
    .with_result_cache()
<<<<<<< HEAD
=======
    // SPEC-103: master EDGEQUAKE_LLM_CACHE default ON → answer cache wired;
    // with_kv_storage upgrades to Tiered L1+L2 on public.llm_cache.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    .with_answer_cache_from_env();

    if let Some(kv) = kv_storage {
        engine = engine.with_kv_storage(kv);
    }

    Arc::new(engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashscope_override_skips_when_not_cross_encoder() {
        let _lock = env_lock();
        let saved_mode = std::env::var("EDGEQUAKE_RERANKER").ok();
        std::env::set_var("EDGEQUAKE_RERANKER", "bm25");
        assert!(try_dashscope_override_reranker().is_none());
        restore_env("EDGEQUAKE_RERANKER", saved_mode);
    }

    #[test]
    fn dashscope_override_builds_when_key_present() {
        // Serialize env mutations — cargo runs lib tests in parallel by default.
        let _lock = env_lock();
        let saved = (
            std::env::var("EDGEQUAKE_RERANKER").ok(),
            std::env::var("EDGEQUAKE_RERANKER_PROVIDER").ok(),
            std::env::var("DASHSCOPE_API_KEY").ok(),
            std::env::var("EDGEQUAKE_RERANKER_MODEL").ok(),
            std::env::var("EDGEQUAKE_RERANKER_BASE_URL").ok(),
        );
        std::env::set_var("EDGEQUAKE_RERANKER", "cross_encoder");
        std::env::set_var("EDGEQUAKE_RERANKER_PROVIDER", "aliyun");
        std::env::set_var("DASHSCOPE_API_KEY", "sk-test-not-fake");
        std::env::set_var("EDGEQUAKE_RERANKER_MODEL", "qwen3-rerank");
        std::env::remove_var("EDGEQUAKE_RERANKER_BASE_URL");
        let r = try_dashscope_override_reranker();
        restore_env("EDGEQUAKE_RERANKER", saved.0);
        restore_env("EDGEQUAKE_RERANKER_PROVIDER", saved.1);
        restore_env("DASHSCOPE_API_KEY", saved.2);
        restore_env("EDGEQUAKE_RERANKER_MODEL", saved.3);
        restore_env("EDGEQUAKE_RERANKER_BASE_URL", saved.4);
        let r = r.expect("override should build");
        assert_eq!(r.model(), "qwen3-rerank");
        assert!(
            r.name().contains("http") || r.name().contains("aliyun") || r.name().contains("rerank")
        );
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn restore_env(name: &str, value: Option<String>) {
        match value {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }
    }
}
