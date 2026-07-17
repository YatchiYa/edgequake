//! SPEC-006 resource runtime — DRY single injection point for AppState.

use std::sync::Arc;

use edgequake_core::{GraphMaterializationSemaphore, PdfVisionSemaphore, ResourceGuard};

use crate::read_path::ReadPathDbPermit;

/// Build shared resource guard + admission semaphores from environment.
pub fn build_resource_runtime() -> (
    ResourceGuard,
    Arc<GraphMaterializationSemaphore>,
    Arc<PdfVisionSemaphore>,
    Arc<ReadPathDbPermit>,
) {
    let guard = ResourceGuard::from_env();
    let graph_materialize = Arc::new(GraphMaterializationSemaphore::from_budget(guard.budget()));
    let pdf_vision = Arc::new(PdfVisionSemaphore::from_budget(guard.budget()));
    let read_path_db = Arc::new(ReadPathDbPermit::from_env());
    (guard, graph_materialize, pdf_vision, read_path_db)
}
