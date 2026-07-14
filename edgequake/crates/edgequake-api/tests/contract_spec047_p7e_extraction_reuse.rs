//! SPEC-047 P7e contract: extraction reuse plan (pure SSOT).

use edgequake_api::processor::pipeline_checkpoint::{
    plan_extraction_reuse, ExtractionReuseKind, ExtractionReusePlan, EXTRACTION_SNAPSHOT_SUFFIX,
};

#[test]
fn contract_p7e_snapshot_suffix() {
    assert_eq!(EXTRACTION_SNAPSHOT_SUFFIX, "-extraction-snapshot");
}

#[test]
fn contract_p7e_plan_prefers_checkpoint_then_snapshot() {
    assert_eq!(
        plan_extraction_reuse(true, true, false, false),
        ExtractionReusePlan::Reuse(ExtractionReuseKind::CrashCheckpoint)
    );
    assert_eq!(
        plan_extraction_reuse(false, true, false, true),
        ExtractionReusePlan::Reuse(ExtractionReuseKind::DurableSnapshot)
    );
}

#[test]
fn contract_p7e_merge_only_fails_closed_without_store() {
    assert_eq!(
        plan_extraction_reuse(false, false, false, true),
        ExtractionReusePlan::MergeOnlyMissing
    );
}

#[test]
fn contract_p7e_force_fresh_clears_reuse() {
    assert_eq!(
        plan_extraction_reuse(true, true, true, false),
        ExtractionReusePlan::Fresh
    );
}
