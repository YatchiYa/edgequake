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
    /// Provider misconfiguration: missing/invalid credentials or an unsupported
    /// runtime provider/model selection. Deterministic within a process — the
    /// operator must fix env/config and restart, so it never resolves on retry.
    /// Distinct from transient `ProviderUnavailable` (network blip, local server
    /// momentarily down).
    ProviderMisconfigured,
    /// User/system cancel — terminal, never retry.
    Cancelled,
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
            Self::ProviderMisconfigured => "provider_misconfigured",
            Self::Cancelled => "cancelled",
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
            Self::ProviderUnavailable => "reduce_concurrency_or_check_provider",
            Self::ProviderMisconfigured => "configure_provider_credentials",
            Self::Cancelled => "none",
            Self::Unknown => "retry",
        }
    }

    /// Permanent failures must not consume the task retry budget (SPEC-045 EC-045-09).
    pub fn is_permanent(self) -> bool {
        matches!(
            self,
            Self::CircuitBreaker
                | Self::DocumentTooLarge
                | Self::EmbeddingLimit
                | Self::GraphMerge
                | Self::ProviderMisconfigured
                | Self::Cancelled
        )
    }
}

/// True when the error is a deterministic provider misconfiguration —
/// missing/invalid credentials or an unsupported runtime provider/model.
///
/// WHY separate from transient `ProviderUnavailable`: a missing `*_API_KEY`,
/// an invalid/incorrect key (HTTP 401), or an unconfigured runtime provider
/// will **never** succeed on retry within the same server process. Retrying
/// only burns the retry budget (exponential backoff) and delays an actionable
/// failure. This must be classified as permanent and surfaced immediately.
///
/// Conservative by design: only fires on explicit configuration/credential
/// markers so genuinely transient "failed to create provider" errors (e.g. a
/// network blip during model discovery) stay retryable as `ProviderUnavailable`.
pub fn is_provider_misconfig_message(error_msg: &str) -> bool {
    let lower = error_msg.to_ascii_lowercase();
    lower.contains("configuration error")
        || lower.contains("api_key is not set")
        || lower.contains("api key is not set")
        || lower.contains("api_key environment variable not set")
        || lower.contains("environment variable not set")
        || lower.contains("is not set. to use")
        || lower.contains("credentials not configured")
        || lower.contains("not configured for this runtime")
        || lower.contains("invalid api key")
        || lower.contains("invalid_api_key")
        || lower.contains("incorrect api key")
        || lower.contains("authentication error")
        || (lower.contains("unauthorized") && lower.contains("api key"))
}

/// True when an error string represents user/system cancel (SPEC-057).
pub fn is_cancel_failure_message(error_msg: &str) -> bool {
    let lower = error_msg.to_ascii_lowercase();
    lower.contains("task cancelled")
        || lower.contains("cancelled by user")
        || lower.contains("cancelled during")
}

/// Classify a permanent failure message into a stable `failure_class` key.
pub fn classify_ingestion_failure(error_msg: &str) -> IngestionFailureClass {
    let lower = error_msg.to_ascii_lowercase();
    if is_cancel_failure_message(error_msg) {
        return IngestionFailureClass::Cancelled;
    }
    // Deterministic credential/config failure — must precede the transient
    // `ProviderUnavailable` branch (which also matches "failed to create").
    if is_provider_misconfig_message(error_msg) {
        return IngestionFailureClass::ProviderMisconfigured;
    }
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
        IngestionFailureClass::ProviderMisconfigured => "provider_config",
        IngestionFailureClass::Cancelled => "cancelled",
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
        assert_eq!(
            class.recommended_action(),
            "reduce_concurrency_or_check_provider"
        );
    }

    #[test]
    fn missing_api_key_is_permanent_misconfig() {
        // Exact message emitted by the workspace pipeline factory + vision path.
        let msg = "Processing error: Failed to create vision provider 'mistral': \
                   Configuration error: MISTRAL_API_KEY is not set. To use the Mistral \
                   provider, set the environment variable and restart the server.";
        let class = classify_ingestion_failure(msg);
        assert_eq!(class, IngestionFailureClass::ProviderMisconfigured);
        assert!(class.is_permanent());
        assert!(is_permanent_ingestion_failure(msg));
        assert_eq!(class.recommended_action(), "configure_provider_credentials");
        assert_eq!(class.as_str(), "provider_misconfigured");
        assert_eq!(failure_step(class), "provider_config");
    }

    #[test]
    fn embedding_env_var_not_set_is_permanent_misconfig() {
        let msg = "Failed to create LLM (Configuration error: MISTRAL_API_KEY is not set.) \
                   and embedding (Configuration error: MISTRAL_API_KEY environment variable \
                   not set. Get your API key from https://console.mistral.ai) providers";
        assert_eq!(
            classify_ingestion_failure(msg),
            IngestionFailureClass::ProviderMisconfigured
        );
        assert!(is_permanent_ingestion_failure(msg));
    }

    #[test]
    fn invalid_api_key_401_is_permanent_misconfig() {
        let msg = "LLM error: Authentication error: invalid_request_error: Incorrect API key \
                   provided (code: invalid_api_key)";
        assert_eq!(
            classify_ingestion_failure(msg),
            IngestionFailureClass::ProviderMisconfigured
        );
        assert!(is_permanent_ingestion_failure(msg));
    }

    #[test]
    fn transient_failed_to_create_provider_stays_retryable() {
        // No credential/config marker → genuinely transient construction failure
        // (e.g. discovery network blip) must remain retryable, not permanent.
        let msg = "Failed to create provider: connection refused";
        let class = classify_ingestion_failure(msg);
        assert_eq!(class, IngestionFailureClass::ProviderUnavailable);
        assert!(!class.is_permanent());
        assert!(!is_permanent_ingestion_failure(msg));
    }

    #[test]
    fn spec045_rate_limit_not_classified_permanent() {
        let msg = "Embedding error: API error: rate limit exceeded (429)";
        assert!(!is_permanent_ingestion_failure(msg));
    }

    #[test]
    fn cancel_is_permanent_non_retryable() {
        let msg = "Task cancelled during 'pre-extraction' stage for document abc";
        let class = classify_ingestion_failure(msg);
        assert_eq!(class, IngestionFailureClass::Cancelled);
        assert!(class.is_permanent());
        assert!(is_permanent_ingestion_failure(msg));
    }

    #[test]
    fn vision_cancel_string_is_cancelled_class() {
        let msg = "Cancelled during vision PDF conversion";
        assert!(is_cancel_failure_message(msg));
        assert_eq!(
            classify_ingestion_failure(msg),
            IngestionFailureClass::Cancelled
        );
        assert!(is_permanent_ingestion_failure(msg));
    }
}
