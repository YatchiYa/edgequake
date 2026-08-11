//! Vision → EdgeParse fallback policy (SPEC-017 P1-09 / SPEC-123).
//!
//! Centralizes when a vision backend failure should degrade to EdgeParse
//! instead of failing the ingestion task outright.
//!
//! # First principles (LAW-123-1 / LAW-123-4)
//!
//! - **Auto** (`backend_explicit=false`): timeout / provider failure may
//!   degrade to EdgeParse so ingestion still completes.
//! - **Resolved Vision** (upload / workspace / tenant / env / default):
//!   choice is law — **fail closed**, no silent EdgeParse. Callers must
//!   surface the error so the UI can retry / switch.

use crate::PdfParserBackend;

/// Classification of vision extraction failures for fallback decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionFailureKind {
    Timeout,
    ProviderUnavailable,
    ConversionFailed,
    FeatureUnavailable,
}

impl VisionFailureKind {
    pub fn as_detail_str(self) -> &'static str {
        match self {
            Self::Timeout => "timed out",
            Self::ProviderUnavailable => "provider unavailable",
            Self::ConversionFailed => "conversion failed",
            Self::FeatureUnavailable => "vision feature unavailable",
        }
    }
}

/// Returns true when a vision backend request should fall back to EdgeParse.
///
/// `backend_explicit`: `true` when workspace or upload explicitly selected
/// Vision (or EdgeParse). Explicit Vision never silently degrades.
pub fn should_fallback_to_edgeparse(
    requested_backend: PdfParserBackend,
    failure: VisionFailureKind,
    backend_explicit: bool,
) -> bool {
    if backend_explicit {
        return false;
    }
    if requested_backend != PdfParserBackend::Vision {
        return false;
    }

    matches!(
        failure,
        VisionFailureKind::Timeout
            | VisionFailureKind::ProviderUnavailable
            | VisionFailureKind::ConversionFailed
            | VisionFailureKind::FeatureUnavailable
    )
}

/// User-visible notice when vision extraction degrades to EdgeParse.
pub fn build_edgeparse_fallback_message(provider: &str, detail: &str) -> String {
    format!(
        "Vision extraction via {provider} was unavailable ({detail}). Falling back to EdgeParse for a more reliable text extraction."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_vision_failures_trigger_edgeparse_fallback() {
        for failure in [
            VisionFailureKind::Timeout,
            VisionFailureKind::ProviderUnavailable,
            VisionFailureKind::ConversionFailed,
            VisionFailureKind::FeatureUnavailable,
        ] {
            assert!(should_fallback_to_edgeparse(
                PdfParserBackend::Vision,
                failure,
                false
            ));
        }
    }

    #[test]
    fn explicit_vision_never_silent_fallback() {
        for failure in [
            VisionFailureKind::Timeout,
            VisionFailureKind::ProviderUnavailable,
            VisionFailureKind::ConversionFailed,
            VisionFailureKind::FeatureUnavailable,
        ] {
            assert!(
                !should_fallback_to_edgeparse(PdfParserBackend::Vision, failure, true),
                "explicit Vision must fail closed for {failure:?}"
            );
        }
    }

    #[test]
    fn edgeparse_requests_do_not_self_fallback() {
        assert!(!should_fallback_to_edgeparse(
            PdfParserBackend::EdgeParse,
            VisionFailureKind::Timeout,
            false
        ));
    }

    #[test]
    fn fallback_message_includes_provider_and_detail() {
        let msg = build_edgeparse_fallback_message("ollama", "timed out");
        assert!(msg.contains("ollama"));
        assert!(msg.contains("timed out"));
        assert!(msg.contains("EdgeParse"));
    }
}
