//! SPEC-119 — graph cleanup discovery timeout messaging (DRY / LAW-119-5).
//!
//! SSOT for detecting source-prefix discovery timeouts and mapping them to
//! product-facing copy. Raw Postgres detail belongs in logs only.

use tracing::warn;

use crate::error::ApiError;
use crate::services::graph_materialization::is_db_statement_timeout;

/// Which product action was running when discovery timed out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphCleanupAction {
    Delete,
    Reprocess,
}

impl GraphCleanupAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Reprocess => "reprocess",
        }
    }
}

/// True when the error is a source-discovery / statement timeout (SPEC-119).
pub fn is_source_discovery_timeout(message: &str) -> bool {
    is_db_statement_timeout(message)
        || message.contains("Source-prefix singular edge")
        || message.contains("Source-prefix edge query")
}

/// User-facing message — no raw Postgres / AGE internals (LAW-119-5).
pub fn graph_cleanup_timeout_user_message(action: GraphCleanupAction) -> String {
    match action {
        GraphCleanupAction::Delete => {
            "Graph cleanup timed out. Retry delete. If it keeps failing, an administrator \
             should verify edge citation indexes (idx_edge_source_chunk_id / \
             idx_edge_source_document_id)."
                .to_string()
        }
        GraphCleanupAction::Reprocess => {
            "Graph cleanup timed out during reprocess. Retry reprocess. If it keeps failing, \
             an administrator should verify edge citation indexes (idx_edge_source_chunk_id / \
             idx_edge_source_document_id)."
                .to_string()
        }
    }
}

/// Deletion worker reason string (still prefixed for existing UI parsers).
pub fn deletion_failed_graph_cleanup_timeout() -> String {
    format!(
        "Deletion failed: {}",
        graph_cleanup_timeout_user_message(GraphCleanupAction::Delete)
    )
}

pub fn api_error_graph_cleanup_timeout(action: GraphCleanupAction) -> ApiError {
    ApiError::ServiceUnavailable {
        message: graph_cleanup_timeout_user_message(action),
        retry_after_secs: 30,
    }
}

/// Log raw detail; keep user message separate.
pub fn log_graph_cleanup_timeout(document_id: &str, action: GraphCleanupAction, detail: &str) {
    warn!(
        document_id = %document_id,
        action = action.as_str(),
        detail = %detail,
        user_message = %graph_cleanup_timeout_user_message(action),
        "SPEC-119: graph cleanup discovery timeout (detail logged, not shown to user)"
    );
}

/// Map a cascade/storage error to a product ApiError when it is a discovery timeout.
pub fn map_cascade_error_for_reprocess(document_id: &str, err: ApiError) -> ApiError {
    let detail = err.to_string();
    if is_source_discovery_timeout(&detail) {
        log_graph_cleanup_timeout(document_id, GraphCleanupAction::Reprocess, &detail);
        api_error_graph_cleanup_timeout(GraphCleanupAction::Reprocess)
    } else {
        err
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_statement_timeout_and_singular_probe() {
        assert!(is_source_discovery_timeout(
            "canceling statement due to statement timeout"
        ));
        assert!(is_source_discovery_timeout(
            "Source-prefix singular edge query failed: boom"
        ));
        assert!(is_source_discovery_timeout(
            "Source-prefix edge query failed: boom"
        ));
        assert!(!is_source_discovery_timeout("connection refused"));
    }

    #[test]
    fn user_message_has_no_raw_postgres() {
        for action in [GraphCleanupAction::Delete, GraphCleanupAction::Reprocess] {
            let msg = graph_cleanup_timeout_user_message(action);
            assert!(!msg.contains("statement timeout"));
            assert!(!msg.contains("canceling statement"));
            assert!(!msg.contains("Source-prefix"));
            assert!(msg.contains("idx_edge_source_chunk_id"));
        }
        let del = deletion_failed_graph_cleanup_timeout();
        assert!(del.starts_with("Deletion failed:"));
        assert!(!del.contains("Detail:"));
    }

    #[test]
    fn map_reprocess_timeout_is_service_unavailable() {
        let err = ApiError::Storage(edgequake_storage::error::StorageError::Database(
            "Source-prefix singular edge query failed: canceling statement due to statement timeout"
                .into(),
        ));
        let mapped = map_cascade_error_for_reprocess("doc-1", err);
        match mapped {
            ApiError::ServiceUnavailable { message, .. } => {
                assert!(message.contains("reprocess"));
                assert!(!message.contains("Source-prefix"));
            }
            other => panic!("expected ServiceUnavailable, got {other:?}"),
        }
    }
}
