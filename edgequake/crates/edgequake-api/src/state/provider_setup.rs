//! Provider setup utilities for application state construction.
//!
//! Provides a single DRY helper to optionally override the embedding provider
//! after [`edgequake_llm::ProviderFactory::from_env()`] with a dedicated host or
//! provider type.  This enables "hybrid mode" where a different service handles
//! LLM inference versus embedding computation.
//!
//! @implements SPEC-140: Separate embedding and chat provider hosts (closes #140)
//!
//! # Environment Variables
//!
//! | Variable | Purpose | Example |
//! |---|---|---|
//! | `EDGEQUAKE_EMBEDDING_PROVIDER` | Override provider type | `ollama`, `openai`, `azure`, `mistral` |
//! | `OLLAMA_EMBEDDING_HOST` | Dedicated Ollama host for embeddings | `http://gpu-box:11434` |
//! | `OLLAMA_EMBEDDING_MODEL` | Model on the dedicated embedding host | `nomic-embed-text` |
//! | `EDGEQUAKE_EMBEDDING_MODEL` | Alternative embedding model override | `text-embedding-3-small` |
//! | `EDGEQUAKE_EMBEDDING_DIMENSION` | Dimension of the embedding vectors | `768`, `1536` |
//! | `EDGEQUAKE_EMBEDDING_BASE_URL` | Override base URL for embedding provider | `https://embed.example.com/v1` |
//! | `EDGEQUAKE_EMBEDDING_API_KEY` | Override API key for embedding provider | `sk-embed-...` |
//! | `AZURE_OPENAI_API_KEY` | Azure embedding (auto-detected when `EDGEQUAKE_EMBEDDING_PROVIDER=azure`) | `sk-...` |
//! | `AZURE_OPENAI_ENDPOINT` | Azure endpoint for embedding | `https://my-resource.openai.azure.com` |
//! | `MISTRAL_API_KEY` | Mistral embedding (auto-detected when `EDGEQUAKE_EMBEDDING_PROVIDER=mistral`) | `...` |

use std::sync::Arc;

use edgequake_llm::traits::EmbeddingProvider;
use edgequake_llm::{OllamaProvider, OpenAIProvider, ProviderFactory};

/// Resolve the embedding provider from environment, optionally overriding the
/// `fallback` returned by `ProviderFactory::from_env()`.
///
/// # Priority
///
/// 1. `EDGEQUAKE_EMBEDDING_PROVIDER` + provider-specific vars → explicit override
/// 2. `OLLAMA_EMBEDDING_HOST` → shortcut to route embeddings to a separate Ollama node
/// 3. `fallback` — the provider already created by `ProviderFactory::from_env()`
///
/// Errors during override creation are logged as warnings and the `fallback` is
/// returned, so startup is never blocked by a misconfigured embedding override.
pub fn resolve_embedding_provider(
    fallback: Arc<dyn EmbeddingProvider>,
) -> Arc<dyn EmbeddingProvider> {
    // --- Priority 1: EDGEQUAKE_EMBEDDING_PROVIDER (explicit provider type) ---
    // WHY: docker-compose may pass an empty string when the host env var is unset
    // (e.g. `EDGEQUAKE_EMBEDDING_PROVIDER: ${EDGEQUAKE_EMBEDDING_PROVIDER:-}`).
    // Treat empty string as "not set" to avoid a spurious warning and fall through
    // to the auto-detection logic below.
    if let Ok(provider_name) = std::env::var("EDGEQUAKE_EMBEDDING_PROVIDER") {
        if !provider_name.is_empty() {
            let model = embedding_model_from_env();
            let dimension = embedding_dimension_from_env();

            // FIX #163: Check for embedding-specific base URL and API key.
            // WHY: In split-provider deployments, chat and embedding traffic go to
            // different servers with different API keys. Without these overrides,
            // both providers share OPENAI_BASE_URL / OPENAI_API_KEY.
            let embed_base_url = std::env::var("EDGEQUAKE_EMBEDDING_BASE_URL").ok();
            let embed_api_key = std::env::var("EDGEQUAKE_EMBEDDING_API_KEY").ok();

            let has_custom_base_url = embed_base_url.is_some();
            let has_custom_api_key = embed_api_key.is_some();
            let is_openai_compatible = matches!(
                provider_name.to_ascii_lowercase().as_str(),
                "openai" | "openai-compatible" | "openai_compatible"
            );

            if is_openai_compatible && (has_custom_base_url || has_custom_api_key) {
                // Use dedicated credentials only for OpenAI-compatible embedding providers.
                let api_key = embed_api_key
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .unwrap_or_default();
                let base_url = embed_base_url.or_else(|| std::env::var("OPENAI_BASE_URL").ok());

                let provider: Arc<dyn EmbeddingProvider> = if let Some(base_url) = base_url {
                    Arc::new(
                        OpenAIProvider::compatible(api_key, base_url).with_embedding_model(&model),
                    )
                } else {
                    Arc::new(OpenAIProvider::new(api_key).with_embedding_model(&model))
                };

                tracing::info!(
                    provider = %provider_name,
                    model = %model,
                    dimension,
                    has_custom_base_url,
                    has_custom_api_key,
                    "Embedding provider overridden with dedicated base URL/API key (FIX #163)"
                );
                return provider;
            }

            match ProviderFactory::create_embedding_provider(&provider_name, &model, dimension) {
                Ok(provider) => {
                    tracing::info!(
                        provider = %provider_name,
                        model = %model,
                        dimension,
                        "Embedding provider overridden via EDGEQUAKE_EMBEDDING_PROVIDER"
                    );
                    return provider;
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        provider = %provider_name,
                        "Failed to create embedding provider from EDGEQUAKE_EMBEDDING_PROVIDER; \
                         using default"
                    );
                }
            }
        }
    }

    // --- Priority 2: OLLAMA_EMBEDDING_HOST (dedicated Ollama embedding node) ---
    if let Ok(embedding_host) = std::env::var("OLLAMA_EMBEDDING_HOST") {
        let model = std::env::var("OLLAMA_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "nomic-embed-text".to_string());

        match OllamaProvider::builder()
            .host(&embedding_host)
            .embedding_model(&model)
            .build()
        {
            Ok(provider) => {
                tracing::info!(
                    host = %embedding_host,
                    model = %model,
                    "Embedding provider overridden via OLLAMA_EMBEDDING_HOST"
                );
                return Arc::new(provider);
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    host = %embedding_host,
                    "Failed to create Ollama embedding provider from OLLAMA_EMBEDDING_HOST; \
                     using default"
                );
            }
        }
    }

    // --- Priority 3: use whatever from_env() already gave us ---
    fallback
}

/// Apply `EDGEQUAKE_CHAT_*` → standard LLM env var aliases.
///
/// FIX #166: Users expect symmetry with `EDGEQUAKE_EMBEDDING_*` naming.
/// This function maps chat-specific env vars to the standard ones used by
/// `ProviderFactory::from_env()`:
///
/// - `EDGEQUAKE_CHAT_BASE_URL` → `OPENAI_BASE_URL` (if not already set)
/// - `EDGEQUAKE_CHAT_API_KEY`  → `OPENAI_API_KEY`  (if not already set)
/// - `EDGEQUAKE_CHAT_MODEL`    → `EDGEQUAKE_LLM_MODEL` (if not already set)
///
/// Must be called BEFORE `ProviderFactory::from_env()`.
pub fn apply_chat_env_aliases() {
    if let Ok(chat_base_url) = std::env::var("EDGEQUAKE_CHAT_BASE_URL") {
        if std::env::var("OPENAI_BASE_URL").is_err() {
            std::env::set_var("OPENAI_BASE_URL", chat_base_url);
        }
    }
    if let Ok(chat_api_key) = std::env::var("EDGEQUAKE_CHAT_API_KEY") {
        if std::env::var("OPENAI_API_KEY").is_err() {
            std::env::set_var("OPENAI_API_KEY", chat_api_key);
        }
    }
    if let Ok(chat_model) = std::env::var("EDGEQUAKE_CHAT_MODEL") {
        if std::env::var("EDGEQUAKE_LLM_MODEL").is_err() {
            std::env::set_var("EDGEQUAKE_LLM_MODEL", chat_model);
        }
    }
}

/// Read the embedding model name from environment variables.
///
/// Checks `OLLAMA_EMBEDDING_MODEL` then `EDGEQUAKE_EMBEDDING_MODEL`, falling
/// back to `"nomic-embed-text"` if neither is set.
fn embedding_model_from_env() -> String {
    std::env::var("OLLAMA_EMBEDDING_MODEL")
        .or_else(|_| std::env::var("EDGEQUAKE_EMBEDDING_MODEL"))
        .unwrap_or_else(|_| "nomic-embed-text".to_string())
}

/// Read the embedding dimension from `EDGEQUAKE_EMBEDDING_DIMENSION`, defaulting
/// to 768 (compatible with most Ollama embedding models).
fn embedding_dimension_from_env() -> usize {
    std::env::var("EDGEQUAKE_EMBEDDING_DIMENSION")
        .ok()
        .and_then(|d| d.parse::<usize>().ok())
        .unwrap_or(768)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_llm::MockProvider;
    use serial_test::serial;

    fn mock_embedding() -> Arc<dyn EmbeddingProvider> {
        Arc::new(MockProvider::new())
    }

    #[test]
    #[serial]
    fn returns_fallback_when_no_env_vars() {
        std::env::remove_var("EDGEQUAKE_EMBEDDING_PROVIDER");
        std::env::remove_var("OLLAMA_EMBEDDING_HOST");

        let fallback = mock_embedding();
        let result = resolve_embedding_provider(fallback.clone());
        assert_eq!(result.name(), "mock");
    }

    #[test]
    #[serial]
    fn returns_fallback_on_unknown_provider() {
        std::env::remove_var("OLLAMA_EMBEDDING_HOST");
        std::env::set_var("EDGEQUAKE_EMBEDDING_PROVIDER", "totally_unknown_provider");

        let fallback = mock_embedding();
        let result = resolve_embedding_provider(fallback);
        assert_eq!(result.name(), "mock");

        std::env::remove_var("EDGEQUAKE_EMBEDDING_PROVIDER");
    }

    #[test]
    #[serial]
    fn ollama_embedding_host_overrides_provider() {
        std::env::remove_var("EDGEQUAKE_EMBEDDING_PROVIDER");
        std::env::set_var("OLLAMA_EMBEDDING_HOST", "http://localhost:11434");
        std::env::set_var("OLLAMA_EMBEDDING_MODEL", "nomic-embed-text");

        let result = resolve_embedding_provider(mock_embedding());
        assert_eq!(result.name(), "ollama");

        std::env::remove_var("OLLAMA_EMBEDDING_HOST");
        std::env::remove_var("OLLAMA_EMBEDDING_MODEL");
    }

    #[test]
    #[serial]
    fn embedding_model_from_env_reads_ollama_first() {
        std::env::remove_var("EDGEQUAKE_EMBEDDING_MODEL");
        std::env::set_var("OLLAMA_EMBEDDING_MODEL", "my-model");
        assert_eq!(embedding_model_from_env(), "my-model");
        std::env::remove_var("OLLAMA_EMBEDDING_MODEL");
    }

    #[test]
    #[serial]
    fn embedding_model_from_env_reads_edgequake_fallback() {
        std::env::remove_var("OLLAMA_EMBEDDING_MODEL");
        std::env::set_var("EDGEQUAKE_EMBEDDING_MODEL", "other-model");
        assert_eq!(embedding_model_from_env(), "other-model");
        std::env::remove_var("EDGEQUAKE_EMBEDDING_MODEL");
    }

    #[test]
    #[serial]
    fn embedding_model_from_env_default() {
        std::env::remove_var("OLLAMA_EMBEDDING_MODEL");
        std::env::remove_var("EDGEQUAKE_EMBEDDING_MODEL");
        assert_eq!(embedding_model_from_env(), "nomic-embed-text");
    }

    #[test]
    #[serial]
    fn embedding_dimension_from_env_parses_value() {
        std::env::remove_var("EDGEQUAKE_EMBEDDING_DIMENSION");
        std::env::set_var("EDGEQUAKE_EMBEDDING_DIMENSION", "1536");
        assert_eq!(embedding_dimension_from_env(), 1536);
        std::env::remove_var("EDGEQUAKE_EMBEDDING_DIMENSION");
    }

    #[test]
    #[serial]
    fn embedding_dimension_from_env_default() {
        std::env::remove_var("EDGEQUAKE_EMBEDDING_DIMENSION");
        assert_eq!(embedding_dimension_from_env(), 768);
    }

    #[test]
    #[serial]
    fn embedding_dimension_from_env_invalid_falls_back() {
        std::env::set_var("EDGEQUAKE_EMBEDDING_DIMENSION", "not_a_number");
        assert_eq!(embedding_dimension_from_env(), 768);
        std::env::remove_var("EDGEQUAKE_EMBEDDING_DIMENSION");
    }

    #[test]
    #[serial]
    fn apply_chat_env_aliases_populates_missing_standard_vars() {
        std::env::remove_var("EDGEQUAKE_CHAT_BASE_URL");
        std::env::remove_var("EDGEQUAKE_CHAT_API_KEY");
        std::env::remove_var("EDGEQUAKE_CHAT_MODEL");
        std::env::remove_var("OPENAI_BASE_URL");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("EDGEQUAKE_LLM_MODEL");

        std::env::set_var("EDGEQUAKE_CHAT_BASE_URL", "https://chat.example.test/v1");
        std::env::set_var("EDGEQUAKE_CHAT_API_KEY", "chat-key");
        std::env::set_var("EDGEQUAKE_CHAT_MODEL", "gpt-test");

        apply_chat_env_aliases();

        assert_eq!(
            std::env::var("OPENAI_BASE_URL").as_deref(),
            Ok("https://chat.example.test/v1")
        );
        assert_eq!(std::env::var("OPENAI_API_KEY").as_deref(), Ok("chat-key"));
        assert_eq!(
            std::env::var("EDGEQUAKE_LLM_MODEL").as_deref(),
            Ok("gpt-test")
        );

        std::env::remove_var("EDGEQUAKE_CHAT_BASE_URL");
        std::env::remove_var("EDGEQUAKE_CHAT_API_KEY");
        std::env::remove_var("EDGEQUAKE_CHAT_MODEL");
        std::env::remove_var("OPENAI_BASE_URL");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("EDGEQUAKE_LLM_MODEL");
    }

    #[test]
    #[serial]
    fn apply_chat_env_aliases_preserves_explicit_standard_vars() {
        std::env::set_var("EDGEQUAKE_CHAT_BASE_URL", "https://chat.example.test/v1");
        std::env::set_var("EDGEQUAKE_CHAT_API_KEY", "chat-key");
        std::env::set_var("EDGEQUAKE_CHAT_MODEL", "gpt-chat");
        std::env::set_var("OPENAI_BASE_URL", "https://explicit.example.test/v1");
        std::env::set_var("OPENAI_API_KEY", "explicit-key");
        std::env::set_var("EDGEQUAKE_LLM_MODEL", "gpt-explicit");

        apply_chat_env_aliases();

        assert_eq!(
            std::env::var("OPENAI_BASE_URL").as_deref(),
            Ok("https://explicit.example.test/v1")
        );
        assert_eq!(
            std::env::var("OPENAI_API_KEY").as_deref(),
            Ok("explicit-key")
        );
        assert_eq!(
            std::env::var("EDGEQUAKE_LLM_MODEL").as_deref(),
            Ok("gpt-explicit")
        );

        std::env::remove_var("EDGEQUAKE_CHAT_BASE_URL");
        std::env::remove_var("EDGEQUAKE_CHAT_API_KEY");
        std::env::remove_var("EDGEQUAKE_CHAT_MODEL");
        std::env::remove_var("OPENAI_BASE_URL");
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("EDGEQUAKE_LLM_MODEL");
    }
}
