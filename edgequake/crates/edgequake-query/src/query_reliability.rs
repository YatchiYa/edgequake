//! Query failure taxonomy (SPEC-045 SRE-Q01).
//!
//! Mirrors `edgequake_tasks::ingestion_reliability` for query pipeline errors.

use serde_json::{json, Value};

/// Typed failure classes for query execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryFailureClass {
    VectorEmpty,
    DimensionMismatch,
    GraphUnavailable,
    LlmAuth,
    Timeout,
    RateLimited,
    NoResults,
    InvalidQuery,
    Unknown,
}

impl QueryFailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VectorEmpty => "vector_empty",
            Self::DimensionMismatch => "dimension_mismatch",
            Self::GraphUnavailable => "graph_unavailable",
            Self::LlmAuth => "llm_auth",
            Self::Timeout => "timeout",
            Self::RateLimited => "rate_limited",
            Self::NoResults => "no_results",
            Self::InvalidQuery => "invalid_query",
            Self::Unknown => "unknown",
        }
    }

    pub fn recommended_action(self) -> &'static str {
        match self {
            Self::VectorEmpty => "reindex_documents",
            Self::DimensionMismatch => "rebuild_embeddings",
            Self::GraphUnavailable => "check_age_extension",
            Self::LlmAuth => "check_api_key",
            Self::Timeout => "retry_simpler_query",
            Self::RateLimited => "retry_later",
            Self::NoResults => "broaden_query",
            Self::InvalidQuery => "fix_query",
            Self::Unknown => "retry",
        }
    }
}

/// Classify a query error message into a stable failure class.
pub fn classify_query_failure(error_msg: &str) -> QueryFailureClass {
    let lower = error_msg.to_ascii_lowercase();
    if lower.contains("dimension mismatch") || lower.contains("cached=") {
        return QueryFailureClass::DimensionMismatch;
    }
    if lower.contains("no results") || lower.contains("no_results") {
        return QueryFailureClass::NoResults;
    }
    if lower.contains("vector") && (lower.contains("empty") || lower.contains("not found")) {
        return QueryFailureClass::VectorEmpty;
    }
    if lower.contains("graph") && (lower.contains("unavailable") || lower.contains("age")) {
        return QueryFailureClass::GraphUnavailable;
    }
    if lower.contains("api key")
        || lower.contains("unauthorized")
        || lower.contains("authentication")
        || lower.contains("401")
    {
        return QueryFailureClass::LlmAuth;
    }
    if lower.contains("timed out") || lower.contains("timeout") {
        return QueryFailureClass::Timeout;
    }
    if lower.contains("rate limit") || lower.contains("429") {
        return QueryFailureClass::RateLimited;
    }
    if lower.contains("invalid query") {
        return QueryFailureClass::InvalidQuery;
    }
    QueryFailureClass::Unknown
}

/// Build diagnostic JSON for API error responses.
pub fn query_failure_diagnostic(error_msg: &str) -> Value {
    let class = classify_query_failure(error_msg);
    json!({
        "failure_class": class.as_str(),
        "recommended_action": class.recommended_action(),
        "message": error_msg,
    })
}

impl QueryFailureClass {
    /// Map from [`crate::error::QueryError`].
    pub fn from_query_error(err: &crate::error::QueryError) -> Self {
        use crate::error::QueryError;
        match err {
            QueryError::InvalidQuery(_) => Self::InvalidQuery,
            QueryError::NoResults => Self::NoResults,
            QueryError::Timeout(_) => Self::Timeout,
            QueryError::StorageError(se) => classify_query_failure(&se.to_string()),
            QueryError::LlmError(le) => classify_query_failure(&le.to_string()),
            QueryError::ConfigError(msg) => classify_query_failure(msg),
            QueryError::Internal(msg) => classify_query_failure(msg),
            QueryError::ContextLimitExceeded { .. } => Self::InvalidQuery,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec045_query_dimension_mismatch() {
        let msg = "Dimension mismatch for workspace: cached=768, requested=1536";
        assert_eq!(
            classify_query_failure(msg),
            QueryFailureClass::DimensionMismatch
        );
        assert_eq!(
            classify_query_failure(msg).recommended_action(),
            "rebuild_embeddings"
        );
    }

    #[test]
    fn spec045_query_llm_auth() {
        let msg = "LLM error: API key invalid (401)";
        assert_eq!(classify_query_failure(msg), QueryFailureClass::LlmAuth);
    }
}
