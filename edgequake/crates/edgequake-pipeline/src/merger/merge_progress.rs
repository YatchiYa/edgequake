//! Merge progress emission helpers (SPEC-032 W-04, SPEC-045 UX).
//!
//! Single place for chunked relationship progress so banner/UI counters advance
//! during long `merge_relationships_batch` runs.

use super::{emit_progress, MergePhase, MergeProgress, MergeProgressCallback};

/// Emit relationship-merge progress every N relationships (tunable via env).
pub fn relationship_merge_chunk_size() -> usize {
    std::env::var("EDGEQUAKE_RELATIONSHIP_MERGE_PROGRESS_CHUNK")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| (8..=512).contains(&n))
        .unwrap_or(64)
}

/// Context passed into batch merge functions for incremental callbacks.
#[derive(Clone, Copy)]
pub struct MergeProgressCtx<'a> {
    pub callback: Option<&'a MergeProgressCallback>,
    pub entities_processed: usize,
    pub entities_total: usize,
    pub relationships_total: usize,
    pub entities_created: usize,
    pub entities_updated: usize,
}

impl<'a> MergeProgressCtx<'a> {
    pub fn emit_relationship_graph(
        &self,
        relationships_processed: usize,
        relationships_created: usize,
        relationships_updated: usize,
    ) {
        emit_progress(
            self.callback,
            MergeProgress {
                phase: MergePhase::RelationshipGraph,
                entities_processed: self.entities_processed,
                entities_total: self.entities_total,
                relationships_processed,
                relationships_total: self.relationships_total,
                entities_created: self.entities_created,
                entities_updated: self.entities_updated,
                relationships_created,
                relationships_updated,
            },
        );
    }
}
