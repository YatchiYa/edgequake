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

/// True when the message carries a **typed** timeout marker (SPEC-083 X-30).
///
/// Prefers [`crate::error::QueryError::Timeout`] Display (`Query timed out after …ms`),
/// `"operation timed out"`, and explicit class tokens — not bare `"timeout"` in
/// arbitrary user/business content.
pub fn is_typed_timeout_message(error_msg: &str) -> bool {
    let lower = error_msg.to_ascii_lowercase();
    lower.starts_with("operation timed out")
        || lower.contains("operation timed out")
        || lower.contains("query timed out after")
        || lower.contains("request timed out")
        || lower.contains("failure_class=timeout")
        || lower.contains("[timeout]")
}

/// True when the message carries a **typed** rate-limit marker (SPEC-083 X-30).
fn is_typed_rate_limited_message(error_msg: &str) -> bool {
    let lower = error_msg.to_ascii_lowercase();
    lower.contains("rate limit")
        || lower.contains("rate_limited")
        || lower.contains("failure_class=rate_limited")
        || lower.contains("[rate_limit]")
        || lower.contains("[rate_limited]")
        // Structured status tokens only — not bare "429" inside prose.
        || lower.contains("http 429")
        || lower.contains("status: 429")
        || lower.contains("status=429")
        || lower.contains("(429)")
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
        || lower.contains("(401)")
        || lower.contains("status=401")
        || lower.contains("http 401")
    {
        return QueryFailureClass::LlmAuth;
    }
    // X-30: typed timeout / rate-limit markers only (no bare "timeout" / "429").
    if is_typed_timeout_message(error_msg) {
        return QueryFailureClass::Timeout;
    }
    if is_typed_rate_limited_message(error_msg) {
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
    ///
    /// Prefer enum arms (X-30); fall back to typed string markers only for
    /// wrapped storage/LLM/config/internal payloads.
    pub fn from_query_error(err: &crate::error::QueryError) -> Self {
        use crate::error::QueryError;
        use edgequake_llm::error::LlmError;
        match err {
            QueryError::InvalidQuery(_) => Self::InvalidQuery,
            QueryError::NoResults => Self::NoResults,
            QueryError::Timeout(_) => Self::Timeout,
            QueryError::ContextLimitExceeded { .. } => Self::InvalidQuery,
            QueryError::LlmError(le) => match le {
                LlmError::Timeout => Self::Timeout,
                LlmError::RateLimited(_) => Self::RateLimited,
                LlmError::AuthError(_) => Self::LlmAuth,
                other => classify_query_failure(&other.to_string()),
            },
            QueryError::StorageError(se) => classify_query_failure(&se.to_string()),
            QueryError::ConfigError(msg) => classify_query_failure(msg),
            QueryError::Internal(msg) => classify_query_failure(msg),
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

    #[test]
    fn unit_failure_class_typed_timeout_and_rate_limit() {
        use crate::error::QueryError;
        use edgequake_llm::error::LlmError;

        assert_eq!(
            QueryFailureClass::from_query_error(&QueryError::Timeout(1500)),
            QueryFailureClass::Timeout
        );
        assert_eq!(
            classify_query_failure("Query timed out after 1500ms"),
            QueryFailureClass::Timeout
        );
        assert_eq!(
            classify_query_failure("Operation timed out"),
            QueryFailureClass::Timeout
        );
        assert_eq!(
            QueryFailureClass::from_query_error(&QueryError::LlmError(LlmError::Timeout)),
            QueryFailureClass::Timeout
        );
        assert_eq!(
            QueryFailureClass::from_query_error(&QueryError::LlmError(LlmError::RateLimited(
                "rpm".into()
            ))),
            QueryFailureClass::RateLimited
        );
        assert_eq!(
            classify_query_failure("upstream rate_limited (429)"),
            QueryFailureClass::RateLimited
        );
    }

    #[test]
    fn unit_breaker_ignores_business_timeout_word() {
        let business = "User asked about query timeout policy in the handbook";
        assert!(!is_typed_timeout_message(business));
        assert_eq!(classify_query_failure(business), QueryFailureClass::Unknown);
        // Bare "429" in prose must not trip rate-limit class.
        assert_eq!(
            classify_query_failure("section 429 of the user guide"),
            QueryFailureClass::Unknown
        );
    }

    #[test]
    fn contract_no_substring_retry_matching() {
        let src = include_str!("query_reliability.rs");
        let prod = src
            .split("#[cfg(test)]")
            .next()
            .expect("production section");
        let bare_timeout = format!("{}{}{}", "contains(", "\"timeout\"", ")");
        let bare_429 = format!("{}{}{}", "contains(", "\"429\"", ")");
        assert!(
            !prod.contains(&bare_timeout),
            "X-30: production classify path must not use bare timeout substring matching"
        );
        assert!(
            !prod.contains(&bare_429),
            "X-30: production classify path must not use bare 429 substring matching"
        );
        assert!(
            prod.contains("from_query_error") && prod.contains("QueryError::Timeout"),
            "prefer from_query_error enum arms for Timeout"
        );
    }
}
