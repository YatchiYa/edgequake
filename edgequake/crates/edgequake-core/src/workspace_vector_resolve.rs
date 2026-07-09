//! SSOT for resolving per-workspace vector storage during ingestion.
//!
//! Shared by the library orchestrator (`EdgeQuake::insert`) and the API worker /
//! upload handlers so vectors land in workspace-scoped tables, not the global default.

use std::sync::Arc;

use edgequake_storage::traits::{VectorStorage, WorkspaceVectorConfig, WorkspaceVectorRegistry};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::workspace_service::WorkspaceService;

/// Canonical UUID for the built-in default workspace (`workspace_id = "default"`).
pub fn default_workspace_uuid() -> Uuid {
    Uuid::from_u128(3)
}

/// Parse a workspace identifier, honoring the legacy `default` alias.
pub fn resolve_workspace_uuid(workspace_id: Option<&str>) -> Option<Uuid> {
    match workspace_id.map(str::trim) {
        None | Some("") => None,
        Some("default") => Some(default_workspace_uuid()),
        Some(value) => Uuid::parse_str(value).ok(),
    }
}

/// Whether ingestion may fall back to the registry default storage on lookup errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceVectorResolvePolicy {
    /// Fail when workspace context cannot be resolved (production ingestion).
    Strict,
    /// Fall back to default storage with a warning (tests / legacy).
    AllowDefaultFallback,
}

/// Inputs for workspace vector resolution during document ingestion.
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceVectorResolveInput<'a> {
    pub workspace_id: Option<&'a str>,
    /// Vector table namespace (API uses `"default"`).
    pub vector_namespace: &'a str,
}

impl<'a> WorkspaceVectorResolveInput<'a> {
    pub fn new(workspace_id: Option<&'a str>, vector_namespace: &'a str) -> Self {
        Self {
            workspace_id,
            vector_namespace,
        }
    }
}

/// True when a registry error indicates stale cached dimension (OODA-225 / SPEC-045 SRE-Q02).
pub fn is_dimension_mismatch_error(error_msg: &str) -> bool {
    error_msg.contains("Dimension mismatch") || error_msg.contains("cached=")
}

/// Resolve workspace-specific vector storage for ingestion writes.
///
/// When [`WorkspaceVectorResolvePolicy::AllowDefaultFallback`] is active and resolution
/// fails, returns `registry.default_storage()` so memory-mode tests keep working.
pub async fn resolve_workspace_vector_storage(
    registry: &dyn WorkspaceVectorRegistry,
    default_storage: Arc<dyn VectorStorage>,
    workspace_service: Option<&dyn WorkspaceService>,
    fallback_embedding_dimension: usize,
    input: WorkspaceVectorResolveInput<'_>,
    policy: WorkspaceVectorResolvePolicy,
) -> Result<Arc<dyn VectorStorage>> {
    let allow_fallback = policy == WorkspaceVectorResolvePolicy::AllowDefaultFallback;

    let workspace_id_raw = input.workspace_id.map(str::trim).unwrap_or("");
    if workspace_id_raw.is_empty() {
        if allow_fallback {
            tracing::warn!("Empty workspace ID — using default vector storage (non-strict mode)");
            return Ok(default_storage);
        }
        return Err(Error::validation(
            "Cannot ingest documents without a valid workspace ID. \
             Please ensure workspace context is properly set.",
        ));
    }

    let workspace_uuid = match resolve_workspace_uuid(Some(workspace_id_raw)) {
        Some(uuid) => uuid,
        None => {
            if allow_fallback {
                tracing::warn!(
                    workspace_id = %workspace_id_raw,
                    "Invalid workspace ID — using default vector storage (non-strict mode)"
                );
                return Ok(default_storage);
            }
            return Err(Error::validation(format!(
                "Invalid workspace ID '{}': could not resolve to a UUID",
                workspace_id_raw
            )));
        }
    };

    let workspace_service = match workspace_service {
        Some(ws) => ws,
        None => {
            if allow_fallback {
                tracing::warn!(
                    workspace_id = %workspace_id_raw,
                    "No workspace service — using default vector storage (non-strict mode)"
                );
                return Ok(default_storage);
            }
            return Err(Error::config(
                "Workspace service not configured. Cannot verify workspace exists.",
            ));
        }
    };

    let workspace = match workspace_service.get_workspace(workspace_uuid).await {
        Ok(Some(ws)) => ws,
        Ok(None) => {
            if allow_fallback {
                tracing::warn!(
                    workspace_id = %workspace_id_raw,
                    "Workspace not found — using default vector storage (non-strict mode)"
                );
                return Ok(default_storage);
            }
            return Err(Error::not_found(format!(
                "Workspace '{}' not found. Cannot ingest documents into non-existent workspace.",
                workspace_id_raw
            )));
        }
        Err(e) => {
            if allow_fallback {
                tracing::warn!(
                    workspace_id = %workspace_id_raw,
                    error = %e,
                    "Failed to lookup workspace — using default vector storage (non-strict mode)"
                );
                return Ok(default_storage);
            }
            return Err(Error::internal(format!(
                "Failed to lookup workspace '{}': {}",
                workspace_id_raw, e
            )));
        }
    };

    let dimension = if workspace.embedding_dimension > 0 {
        workspace.embedding_dimension
    } else {
        fallback_embedding_dimension
    };

    // SPEC-045 SRE-Q02: validate cache hit dimension (parity with query path OODA-225).
    if let Some(storage) = registry.get(&workspace_uuid).await {
        let cached_dim = storage.dimension();
        if cached_dim != dimension {
            tracing::warn!(
                workspace_id = %workspace_id_raw,
                cached_dim = cached_dim,
                requested_dim = dimension,
                "Ingestion dimension mismatch on cache hit — evicting and re-resolving"
            );
            registry.evict(&workspace_uuid).await;
        } else {
            return Ok(storage);
        }
    }

    let config = WorkspaceVectorConfig {
        workspace_id: workspace_uuid,
        dimension,
        namespace: input.vector_namespace.to_string(),
    };

    tracing::debug!(
        workspace_id = %workspace_id_raw,
        dimension = dimension,
        embedding_model = %workspace.embedding_model,
        "Using workspace-specific vector storage for ingestion"
    );

    match registry.get_or_create(config.clone()).await {
        Ok(storage) => Ok(storage),
        Err(e) => {
            let error_msg = e.to_string();
            if is_dimension_mismatch_error(&error_msg) {
                tracing::warn!(
                    workspace_id = %workspace_id_raw,
                    error = %error_msg,
                    "Ingestion dimension mismatch on get_or_create — evicting cache and retrying"
                );
                registry.evict(&workspace_uuid).await;
                return match registry.get_or_create(config).await {
                    Ok(storage) => Ok(storage),
                    Err(e2) if allow_fallback => {
                        tracing::warn!(
                            workspace_id = %workspace_id_raw,
                            error = %e2,
                            "Retry after eviction failed — using default (non-strict mode)"
                        );
                        Ok(default_storage)
                    }
                    Err(e2) => Err(Error::internal(format!(
                        "Failed to create vector storage for workspace '{}' after cache eviction: {}",
                        workspace_id_raw, e2
                    ))),
                };
            }
            if allow_fallback {
                tracing::warn!(
                    workspace_id = %workspace_id_raw,
                    dimension = dimension,
                    error = %error_msg,
                    "Failed to create workspace vector storage — using default (non-strict mode)"
                );
                Ok(default_storage)
            } else {
                Err(Error::internal(format!(
                    "Failed to create vector storage for workspace '{}' (dimension {}): {}",
                    workspace_id_raw, dimension, error_msg
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec045_dimension_mismatch_detector_matches_registry_error() {
        let msg = "Dimension mismatch for workspace abc: cached=768, requested=1536. Clear cache";
        assert!(is_dimension_mismatch_error(msg));
        assert!(is_dimension_mismatch_error("cached=768"));
        assert!(!is_dimension_mismatch_error("connection refused"));
    }
}
