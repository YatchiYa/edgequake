//! SPEC-091 Wave-0 scorecard — baseline metrics harness.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ANN retrieval baseline (Wave 0).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AnnMetrics {
    pub recall_at_10: f64,
    pub p95_latency_ms: f64,
    pub ef_construction: u32,
    pub storage_mode: String,
}

/// Ingestion path baseline.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IngestionMetrics {
    pub p95_ms: f64,
    pub chunks_per_doc_avg: f64,
    pub kv_chunk_keys: u64,
    pub relational_chunk_rows: u64,
}

/// Full-text search baseline.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FullTextMetrics {
    pub p95_latency_ms: f64,
    pub gin_enabled: bool,
}

/// Environment metadata captured once per scorecard run.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ScorecardEnvironment {
    pub hardware: String,
    pub postgres_version: String,
    pub pgvector_version: String,
    pub dataset_shape: String,
    pub concurrency: u32,
    pub cache_state: String,
}

/// Wave-0 scorecard aggregate.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Scorecard {
    pub recorded_at: DateTime<Utc>,
    pub environment: ScorecardEnvironment,
    pub ann: AnnMetrics,
    pub ingestion: IngestionMetrics,
    pub full_text: FullTextMetrics,
}

/// Records and serializes Wave-0 baseline metrics.
#[derive(Debug, Default)]
pub struct ScorecardRecorder {
    scorecard: Scorecard,
}

impl ScorecardRecorder {
    pub fn new(environment: ScorecardEnvironment) -> Self {
        Self {
            scorecard: Scorecard {
                recorded_at: Utc::now(),
                environment,
                ..Default::default()
            },
        }
    }

    pub fn with_ann(mut self, ann: AnnMetrics) -> Self {
        self.scorecard.ann = ann;
        self
    }

    pub fn with_ingestion(mut self, ingestion: IngestionMetrics) -> Self {
        self.scorecard.ingestion = ingestion;
        self
    }

    pub fn with_full_text(mut self, full_text: FullTextMetrics) -> Self {
        self.scorecard.full_text = full_text;
        self
    }

    pub fn scorecard(&self) -> &Scorecard {
        &self.scorecard
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.scorecard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_scorecard_serializes_json() {
        let recorder = ScorecardRecorder::new(ScorecardEnvironment {
            hardware: "local".into(),
            postgres_version: "16".into(),
            pgvector_version: "0.8.5".into(),
            dataset_shape: "smoke".into(),
            concurrency: 1,
            cache_state: "cold".into(),
        })
        .with_ann(AnnMetrics {
            recall_at_10: 0.95,
            p95_latency_ms: 12.0,
            ef_construction: 128,
            storage_mode: "half".into(),
        });
        let json = recorder.to_json().expect("json");
        assert!(json.contains("recall_at_10"));
        assert!(json.contains("hardware"));
    }
}
