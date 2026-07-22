//! SPEC-059: retract searchable indexes whenever a task/doc is cancelled.
//!
//! DRY helper shared by cancel_facade, pipeline cancel, and PDF cancel so
//! unindex does not depend on a live worker hitting `check_cancelled`.

use std::sync::Arc;

use edgequake_storage::traits::{GraphStorage, VectorStorage};
use edgequake_tasks::Task;

use super::retract_document_indexes::retract_document_indexes;
use super::task_document_sync::extract_document_id_from_task;

/// Retract ANN/graph indexes for the document linked to `task` (best-effort).
pub async fn retract_indexes_for_task(
    graph: &Arc<dyn GraphStorage>,
    vector: &Arc<dyn VectorStorage>,
    task: &Task,
) {
    let Some(document_id) = extract_document_id_from_task(task) else {
        return;
    };
    retract_indexes_for_document(graph, vector, &document_id).await;
}

/// Retract ANN/graph indexes for an explicit document id (best-effort).
pub async fn retract_indexes_for_document(
    graph: &Arc<dyn GraphStorage>,
    vector: &Arc<dyn VectorStorage>,
    document_id: &str,
) {
    let stats = retract_document_indexes(graph, vector, None, document_id).await;
    tracing::info!(
        document_id = %document_id,
        embeddings_deleted = stats.embeddings_deleted,
        entities_removed = stats.entities_removed,
        entities_updated = stats.entities_updated,
        "SPEC-059: retracted indexes after cancel"
    );
}
