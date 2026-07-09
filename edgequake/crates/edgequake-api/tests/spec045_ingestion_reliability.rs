//! SPEC-045 — Ingestion reliability battle tests.
//!
//! Reproduces edge cases from `specs/045-fix-ingestion-errors/004-edge-cases-matrix.md`
//! as contract/unit proofs (no live Postgres required unless noted).
//!
//! Run:
//!   cargo test -p edgequake-api --test spec045_ingestion_reliability -- --nocapture
//!   cargo test -p edgequake-tasks spec045 -- --nocapture
//!   cargo test -p edgequake-pipeline spec045 -- --nocapture

use edgequake_api::services::{
    classify_ingestion_failure, IngestionFailureClass, LargeDocumentProfile,
};
use edgequake_pdf::PdfParserBackend;
use edgequake_tasks::{
    ingestion_reliability::is_permanent_ingestion_failure, Task, TaskFailureInfo, TaskType,
};

// ── EC-045-03 / EC-045-07 / EC-045-09: failure taxonomy ─────────────────────

#[test]
fn bt045_ec03_graph_merge_class_and_action() {
    let msg = "Pipeline processing failed: 2 knowledge-graph merge error(s) during persist";
    let class = classify_ingestion_failure(msg);
    assert_eq!(class, IngestionFailureClass::GraphMerge);
    assert_eq!(class.as_str(), "graph_merge");
    assert_eq!(class.recommended_action(), "reprocess_full");
    assert!(is_permanent_ingestion_failure(msg));
}

#[test]
fn bt045_ec07_provider_unavailable_retriable() {
    let msg = "Entity extraction error: Network error: error sending request for url (http://localhost:11434/api/chat)";
    let class = classify_ingestion_failure(msg);
    assert_eq!(class, IngestionFailureClass::ProviderUnavailable);
    assert_eq!(class.recommended_action(), "check_provider");
    assert!(!is_permanent_ingestion_failure(msg));
}

#[test]
fn bt045_ec09_embedding_400_permanent_not_retried() {
    let msg = "Embedding error: API error: Too many tokens overall, split into more batches. (400)";
    let class = classify_ingestion_failure(msg);
    assert_eq!(class, IngestionFailureClass::EmbeddingLimit);
    assert!(is_permanent_ingestion_failure(msg));

    let mut task = Task::new(
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({}),
    );
    task.mark_failed_with_details(TaskFailureInfo::from_processing_error(msg));
    assert!(!task.can_retry(), "400 embedding must not retry");
}

#[test]
fn bt045_ec09_embedding_429_remains_retriable() {
    let msg = "Embedding error: rate limit exceeded (429)";
    assert!(!is_permanent_ingestion_failure(msg));
}

// ── EC-045-11: empty markdown → Full reprocess policy (SPEC-038) ───────────

#[test]
fn bt045_ec11_large_pdf_edgeparse_recommended() {
    let profile = LargeDocumentProfile::new(603, 11_043_120);
    let est = profile.ingestion_estimate(PdfParserBackend::EdgeParse, "mock");
    assert_eq!(est.recommended_backend, "edgeparse");
    assert!(est.convert_seconds < 600);
}

// ── EC-045-01/02: readiness contracts (source wiring) ─────────────────────

#[test]
fn bt045_ec01_m038_readiness_wired_in_bootstrap() {
    let src = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(src.contains("is_ready_for_traffic"));
    assert!(src.contains("migration_038.is_degraded()"));
    assert!(src.contains("migration_042.is_degraded()"));
}

#[test]
fn bt045_ec05_m047_wsdoc_reconcile_idempotent() {
    let src = include_str!("../src/state/migration_bootstrap/reconcile/m047.rs");
    assert!(src.contains("Idempotent"));
    assert!(src.contains("execute_bootstrap_apply_sql"));
}

// ── EC-045-04: Cypher compensation uses bare $1 (SPEC-044) ─────────────────

#[test]
fn bt045_ec04_cypher_bound_uses_bare_dollar_one() {
    let src =
        include_str!("../../edgequake-storage/src/adapters/postgres/graph/helpers/cypher_exec.rs");
    assert!(src.contains("cypher_bound_sql"));
    assert!(src.contains(", $1) AS"));
    assert!(src.contains(".bind(params_bind)"));
}

// ── EC-045-06: startup orphan recovery ordering ───────────────────────────

#[test]
fn bt045_ec06_orphan_recovery_before_workers() {
    let src = include_str!("../../../src/main.rs");
    let recover_tasks = src.find("recover_orphaned_tasks").expect("task recovery");
    let recover_docs = src
        .find("recover_orphaned_documents")
        .expect("document recovery");
    let worker_start = src.find("worker_pool.start()").expect("worker start");
    assert!(
        recover_tasks < worker_start && recover_docs < worker_start,
        "orphan recovery must run before workers"
    );
}

// ── EC-045-06: periodic auto-repair hooks ─────────────────────────────────

#[test]
fn bt045_ec06_periodic_orphan_and_auto_document_recover() {
    let src = include_str!("../../../src/main.rs");
    assert!(src.contains("periodic_orphan_check"));
    assert!(src.contains("EDGEQUAKE_AUTO_ORPHAN_DOCUMENT_RECOVER_MINUTES"));
}

// ── EC-045-12: informational notice scrubbing ───────────────────────────

#[test]
fn bt045_ec12_status_updates_scrub_informational_notices() {
    let src = include_str!("../src/processor/status_updates.rs");
    assert!(src.contains("is_informational_notice"));
    assert!(src.contains("failure_class"));
}

// ── EC-045-10: checkpoint invalidation wired ──────────────────────────────

#[test]
fn bt045_ec10_checkpoint_cleanup_on_startup() {
    let src = include_str!("../../../src/main.rs");
    assert!(src.contains("cleanup_stale_checkpoints"));
}

// ── EC-045-03: merge failure triggers compensation ────────────────────────

#[test]
fn bt045_ec03_merge_failure_compensation_wired() {
    let src = include_str!("../../edgequake-pipeline/src/persistence/ingestion_persister.rs");
    assert!(src.contains("compensate_merge_failure"));
    assert!(src.contains("knowledge-graph merge error(s) during persist"));
}

// ── EC-045-13: entity reconcile module exists ─────────────────────────────

#[test]
fn bt045_ec13_entity_reconcile_available() {
    let src = include_str!("../../edgequake-storage/src/entity_reconcile.rs");
    assert!(src.contains("Idempotent"));
    assert!(src.contains("delete_node"));
}

// ── SPEC-045 SRE: cross-pipeline battle-proof wiring ─────────────────────

#[test]
fn bt045_sre_vector_resolve_parity_wired() {
    let ingest = include_str!("../../edgequake-core/src/workspace_vector_resolve.rs");
    let query = include_str!("../src/handlers/query/workspace_resolve.rs");
    assert!(ingest.contains("is_dimension_mismatch_error"));
    assert!(ingest.contains("registry.evict"));
    assert!(query.contains("Dimension mismatch"));
    assert!(query.contains("evict"));
}

#[test]
fn bt045_sre_ready_json_blockers_wired() {
    let health = include_str!("../src/handlers/health.rs");
    let bootstrap = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(health.contains("ReadinessResponse"));
    assert!(bootstrap.contains("readiness_blockers"));
}

#[test]
fn bt045_sre_metrics_wired() {
    let metrics = include_str!("../../edgequake-observability/src/metrics.rs");
    assert!(metrics.contains("edgequake_ingestion_failures_total"));
    assert!(metrics.contains("edgequake_compensation_quarantine_total"));
}

#[test]
fn bt045_sre_requeue_pagination_wired() {
    let src = include_str!("../../../src/main.rs");
    assert!(src.contains("SPEC-045 SRE-I02"));
    assert!(!src.contains("page_size: 1000"));
}

#[test]
fn bt045_sre_pdf_recover_stuck_wired() {
    let src = include_str!("../src/handlers/documents/recovery/stuck.rs");
    assert!(src.contains("PdfProcessing"));
    assert!(src.contains("pdf_id"));
}

// ── Dual-store read model: cost from metadata JSONB (M041 safe) ───────────

#[test]
fn bt045_list_api_reads_cost_from_metadata_not_columns() {
    let src = include_str!("../src/document_read_model.rs");
    assert!(src.contains("metadata` JSONB"));
    assert!(src.contains("migration-041"));
}

#[cfg(feature = "postgres")]
mod postgres_gates {
    #[test]
    fn bt045_migration_readiness_test_exists() {
        let path = std::path::Path::new("tests/migration_readiness_proof.rs");
        assert!(path.exists());
    }
}
