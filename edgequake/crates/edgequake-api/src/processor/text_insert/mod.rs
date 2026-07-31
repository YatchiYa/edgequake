//! Text insert worker pipeline (SPEC-025 6.6 SRP split).

mod cancel;
mod extraction;
mod finalize;
mod persist;
mod prepare;
mod types;

use super::*;
use edgequake_tasks::FairnessPermit;
use tokio_util::sync::CancellationToken;

impl DocumentTaskProcessor {
    pub(super) async fn process_text_insert(
        &self,
        task: &mut Task,
        data: TextInsertData,
        cancel_token: CancellationToken,
        mut fairness: Option<FairnessPermit>,
    ) -> TaskResult<serde_json::Value> {
        let processing_start = std::time::Instant::now();
        let stage_t0 = std::time::Instant::now();
        let prepared = self
            .text_insert_prepare(task, data, cancel_token.clone())
            .await?;
        edgequake_observability::metrics::record_ingest_stage_duration(
            "prepare",
            stage_t0.elapsed().as_secs_f64(),
        );

        let stage_t0 = std::time::Instant::now();
        let extracted = self
            .text_insert_extract(task, prepared, cancel_token.clone())
            .await?;
        edgequake_observability::metrics::record_ingest_stage_duration(
            "extract",
            stage_t0.elapsed().as_secs_f64(),
        );

        let stage_t0 = std::time::Instant::now();
        let persisted = self
            .text_insert_persist(task, extracted, cancel_token.clone(), &mut fairness)
            .await?;
        // Permit should already be dropped inside persist; drop any remainder.
        drop(fairness);
        edgequake_observability::metrics::record_ingest_stage_duration(
            "persist",
            stage_t0.elapsed().as_secs_f64(),
        );

        let stage_t0 = std::time::Instant::now();
        let out = self
            .text_insert_finalize(persisted, cancel_token, processing_start)
            .await;
        edgequake_observability::metrics::record_ingest_stage_duration(
            "finalize",
            stage_t0.elapsed().as_secs_f64(),
        );
        out
    }
}
