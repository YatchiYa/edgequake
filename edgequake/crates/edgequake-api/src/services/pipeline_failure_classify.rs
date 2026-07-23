//! Typed PipelineError → IngestionFailureClass (SPEC-083 X-30).
//!
//! edgequake-tasks cannot depend on edgequake-pipeline (dependency graph).
//! Typed classification lives here; string paths consume `failure_class=` markers.

use edgequake_llm::error::LlmError;
use edgequake_pipeline::error::PipelineError;
use edgequake_tasks::{classify_ingestion_failure, IngestionFailureClass};

/// Map a [`PipelineError`] / nested [`LlmError`] to [`IngestionFailureClass`].
///
/// Prefer enum arms; fall back to marker/string taxonomy only for Unknown wrapping.
pub fn classify_from_pipeline_error(err: &PipelineError) -> IngestionFailureClass {
    if let Some(token) = err.ingestion_failure_class_token() {
        if let Some(class) = IngestionFailureClass::from_token(token) {
            return class;
        }
    }
    match err {
        PipelineError::LlmError(le) => classify_from_llm_error(le),
        // String-wrapped variants: last-resort taxonomy on Display (+ markers).
        other => classify_ingestion_failure(&other.display_with_failure_class()),
    }
}

/// Map a typed [`LlmError`] to ingestion failure class (X-30).
pub fn classify_from_llm_error(err: &LlmError) -> IngestionFailureClass {
    match err {
        LlmError::Timeout => IngestionFailureClass::TimeoutPhaseExtract,
        LlmError::RateLimited(_) | LlmError::NetworkError(_) => {
            IngestionFailureClass::ProviderUnavailable
        }
        LlmError::AuthError(_) | LlmError::ConfigError(_) => {
            IngestionFailureClass::ProviderMisconfigured
        }
        LlmError::TokenLimitExceeded { .. } => IngestionFailureClass::EmbeddingLimit,
        LlmError::InvalidRequest(msg) => {
            let lower = msg.to_ascii_lowercase();
            if lower.contains("too many inputs") || lower.contains("too many tokens") {
                IngestionFailureClass::EmbeddingLimit
            } else {
                classify_ingestion_failure(msg)
            }
        }
        other => classify_ingestion_failure(&other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_failure_class_typed_from_pipeline_error() {
        assert_eq!(
            classify_from_pipeline_error(&PipelineError::CircuitBreakerOpen {
                failures: 3,
                retry_after_secs: 30,
            }),
            IngestionFailureClass::CircuitBreaker
        );
        assert_eq!(
            classify_from_pipeline_error(&PipelineError::ExtractionTimeout {
                chunk_index: 0,
                timeout_secs: 120,
                message: "hung".into(),
            }),
            IngestionFailureClass::TimeoutPhaseExtract
        );
        assert_eq!(
            classify_from_pipeline_error(&PipelineError::ExtractionTimeout {
                chunk_index: 0,
                timeout_secs: 120,
                message: "vision convert stalled".into(),
            }),
            IngestionFailureClass::TimeoutPhaseConvert
        );
        assert_eq!(
            classify_from_pipeline_error(&PipelineError::LlmError(LlmError::Timeout)),
            IngestionFailureClass::TimeoutPhaseExtract
        );
        assert_eq!(
            classify_from_pipeline_error(&PipelineError::LlmError(LlmError::RateLimited(
                "rpm".into()
            ))),
            IngestionFailureClass::ProviderUnavailable
        );
    }

    #[test]
    fn unit_breaker_ignores_business_timeout_word() {
        // String-wrapped Unknown path must not trip on business prose.
        let err = PipelineError::ExtractionError(
            "Entity description: the project timeout policy is 30 days".into(),
        );
        assert_eq!(
            classify_from_pipeline_error(&err),
            IngestionFailureClass::Unknown
        );
    }
}
