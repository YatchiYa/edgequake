//! Ingestion failure taxonomy and retry policy (SPEC-045 SSOT).
//!
//! Single source for classifying permanent ingestion errors so task workers
//! do not waste retry budget on deterministic failures (embedding 400, graph merge).

/// Typed failure classes for document ingestion (SPEC-045 SSOT).
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestionFailureClass {
    TimeoutPhaseConvert,
    TimeoutPhaseExtract,
    CircuitBreaker,
    DocumentTooLarge,
    EmbeddingLimit,
    GraphMerge,
    ProviderUnavailable,
    Unknown,
}

impl IngestionFailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TimeoutPhaseConvert => "timeout_phase_convert",
            Self::TimeoutPhaseExtract => "timeout_phase_extract",
            Self::CircuitBreaker => "circuit_breaker",
            Self::DocumentTooLarge => "document_too_large",
            Self::EmbeddingLimit => "embedding_limit",
            Self::GraphMerge => "graph_merge",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::Unknown => "unknown",
        }
    }

    pub fn recommended_action(self) -> &'static str {
        match self {
            Self::TimeoutPhaseConvert => "reprocess_edgeparse",
            Self::TimeoutPhaseExtract => "retry_faster_model",
            Self::CircuitBreaker => "reprocess_edgeparse",
            Self::DocumentTooLarge => "split_document",
            Self::EmbeddingLimit => "retry_or_support",
            Self::GraphMerge => "reprocess_full",
            Self::ProviderUnavailable => "check_provider",
            Self::Unknown => "retry",
        }
    }

    /// Permanent failures must not consume the task retry budget (SPEC-045 EC-045-09).
    pub fn is_permanent(self) -> bool {
        matches!(
            self,
            Self::CircuitBreaker | Self::DocumentTooLarge | Self::EmbeddingLimit | Self::GraphMerge
        )
    }
}

/// Classify a permanent failure message into a stable `failure_class` key.
pub fn classify_ingestion_failure(error_msg: &str) -> IngestionFailureClass {
    let lower = error_msg.to_ascii_lowercase();
    if lower.contains("circuit breaker") {
        return IngestionFailureClass::CircuitBreaker;
    }
    if lower.contains("document too large") || lower.contains("exceeds maximum size") {
        return IngestionFailureClass::DocumentTooLarge;
    }
    if lower.contains("too many inputs")
        || lower.contains("too many tokens")
        || lower.contains("invalid_request_prompt")
        || (lower.contains("embedding") && lower.contains("400"))
    {
        return IngestionFailureClass::EmbeddingLimit;
    }
    if lower.contains("knowledge-graph merge error")
        || lower.contains("merge error(s) during persist")
        || (lower.contains("graph error") && lower.contains("merge"))
    {
        return IngestionFailureClass::GraphMerge;
    }
    if lower.contains("timed out") || lower.contains("timeout") {
        if lower.contains("vision") || lower.contains("convert") || lower.contains("markdown") {
            return IngestionFailureClass::TimeoutPhaseConvert;
        }
        return IngestionFailureClass::TimeoutPhaseExtract;
    }
    if lower.contains("provider")
        && (lower.contains("unavailable") || lower.contains("failed to create"))
    {
        return IngestionFailureClass::ProviderUnavailable;
    }
    if lower.contains("network error")
        || lower.contains("connection refused")
        || lower.contains("error sending request")
        || lower.contains("localhost:11434")
    {
        return IngestionFailureClass::ProviderUnavailable;
    }
    IngestionFailureClass::Unknown
}

/// True when the error will not resolve by retrying the same request.
pub fn is_permanent_ingestion_failure(error_msg: &str) -> bool {
    classify_ingestion_failure(error_msg).is_permanent()
        || error_msg
            .to_ascii_lowercase()
            .contains("invalid_request_prompt")
}

/// Map failure class to task pipeline step for structured errors.
pub fn failure_step(class: IngestionFailureClass) -> &'static str {
    match class {
        IngestionFailureClass::EmbeddingLimit => "embedding",
        IngestionFailureClass::GraphMerge => "indexing",
        IngestionFailureClass::TimeoutPhaseConvert => "pdf_convert",
        IngestionFailureClass::TimeoutPhaseExtract | IngestionFailureClass::CircuitBreaker => {
            "extraction"
        }
        IngestionFailureClass::DocumentTooLarge => "admission",
        IngestionFailureClass::ProviderUnavailable => "extraction",
        IngestionFailureClass::Unknown => "processing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec045_graph_merge_is_permanent() {
        let msg = "Pipeline processing failed: 1 knowledge-graph merge error(s) during persist";
        let class = classify_ingestion_failure(msg);
        assert_eq!(class, IngestionFailureClass::GraphMerge);
        assert!(class.is_permanent());
        assert_eq!(class.recommended_action(), "reprocess_full");
    }

    #[test]
    fn spec045_embedding_400_is_permanent() {
        let msg = "Embedding error: API error: Too many inputs in request (400)";
        let class = classify_ingestion_failure(msg);
        assert_eq!(class, IngestionFailureClass::EmbeddingLimit);
        assert!(is_permanent_ingestion_failure(msg));
    }

    #[test]
    fn spec045_provider_error_is_retriable() {
        let msg = "Network error: error sending request for url (http://localhost:11434/api/chat)";
        let class = classify_ingestion_failure(msg);
        assert_eq!(class, IngestionFailureClass::ProviderUnavailable);
        assert!(!class.is_permanent());
    }

    #[test]
    fn spec045_rate_limit_not_classified_permanent() {
        let msg = "Embedding error: API error: rate limit exceeded (429)";
        assert!(!is_permanent_ingestion_failure(msg));
    }
}
