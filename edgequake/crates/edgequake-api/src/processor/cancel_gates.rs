//! SPEC-091 WP1 (LAW-Q7 / LAW-WP): cancel stage-boundary SSOT.
//!
//! Every cooperative cancel check must use a [`CancelGate`] id. Unknown stage
//! strings are rejected in tests and logged as errors in production so drift
//! cannot silently invent new boundaries.

use std::fmt;
use std::str::FromStr;

/// Exhaustive cancel checkpoints for ingest / lifecycle processors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancelGate {
    /// Worker skipped claim after cancel intent (claim loop).
    Claim,
    /// Fairness park waiter aborted.
    Park,
    /// Text-insert prepare (metadata / shell).
    PrePrepare,
    /// Before expensive LLM extraction.
    PreExtraction,
    /// After extraction, before embed / persist.
    PostExtraction,
    /// Before embedding generation / ensure_embeddings.
    PreEmbed,
    /// Before graph/vector materialize (`pre-graph-storage`).
    PreMaterialize,
    /// Before promote / finalize.
    PrePromote,
    /// PDF: before vision convert.
    PreVisionExtraction,
    /// PDF: before enqueue Insert after convert.
    PreIngestEnqueue,
    /// PDF: resume-from-markdown before enqueue.
    PreIngestEnqueueResume,
    /// Knowledge injection gates.
    PreInjection,
    PrePipeline,
}

impl CancelGate {
    /// Wire / log id (stable string used by `check_cancelled`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claim => "claim",
            Self::Park => "park",
            Self::PrePrepare => "pre-prepare",
            Self::PreExtraction => "pre-extraction",
            Self::PostExtraction => "post-extraction",
            Self::PreEmbed => "pre-embed",
            Self::PreMaterialize => "pre-graph-storage",
            Self::PrePromote => "pre-promote",
            Self::PreVisionExtraction => "pre-vision-extraction",
            Self::PreIngestEnqueue => "pre-ingest-enqueue",
            Self::PreIngestEnqueueResume => "pre-ingest-enqueue-resume",
            Self::PreInjection => "pre-injection",
            Self::PrePipeline => "pre-pipeline",
        }
    }

    /// All gates (conformance / WP-AC-04).
    pub const ALL: &'static [CancelGate] = &[
        Self::Claim,
        Self::Park,
        Self::PrePrepare,
        Self::PreExtraction,
        Self::PostExtraction,
        Self::PreEmbed,
        Self::PreMaterialize,
        Self::PrePromote,
        Self::PreVisionExtraction,
        Self::PreIngestEnqueue,
        Self::PreIngestEnqueueResume,
        Self::PreInjection,
        Self::PrePipeline,
    ];

    /// Parse a stage string; returns error for unknown ids.
    pub fn parse(stage: &str) -> Result<Self, UnknownCancelGate> {
        stage.parse()
    }

    /// Validate stage string; in test builds panic on unknown, else warn.
    pub fn assert_known(stage: &str) -> &str {
        match Self::parse(stage) {
            Ok(g) => g.as_str(),
            Err(err) => {
                #[cfg(test)]
                panic!("unknown cancel gate: {err}");
                #[cfg(not(test))]
                {
                    tracing::error!(
                        stage = %stage,
                        error = %err,
                        "SPEC-091 WP1: unknown cancel gate (not in CancelGate SSOT)"
                    );
                    stage
                }
            }
        }
    }
}

impl fmt::Display for CancelGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CancelGate {
    type Err = UnknownCancelGate;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for g in Self::ALL {
            if g.as_str() == s {
                return Ok(*g);
            }
        }
        // Legacy aliases still accepted → canonical PreMaterialize.
        if s == "pre-lineage" || s == "post-lineage" {
            return Ok(Self::PreMaterialize);
        }
        Err(UnknownCancelGate(s.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCancelGate(pub String);

impl fmt::Display for UnknownCancelGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown cancel gate {:?}", self.0)
    }
}

impl std::error::Error for UnknownCancelGate {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_gates_roundtrip() {
        for g in CancelGate::ALL {
            assert_eq!(CancelGate::parse(g.as_str()).unwrap(), *g);
        }
    }

    #[test]
    fn unknown_rejected() {
        assert!(CancelGate::parse("not-a-gate").is_err());
    }

    #[test]
    fn legacy_lineage_aliases_map_to_materialize() {
        assert_eq!(
            CancelGate::parse("pre-lineage").unwrap(),
            CancelGate::PreMaterialize
        );
    }
}
