use super::super::*;
use tokio_util::sync::CancellationToken;

impl DocumentTaskProcessor {
    /// Stages that run after graph/vector persist — cancellation here leaves usable data.
    fn is_post_graph_stage(stage: &str) -> bool {
        matches!(stage, "pre-lineage" | "post-lineage")
    }

    /// Check if the task has been cancelled and return early if so.
    pub(crate) async fn check_cancelled(
        &self,
        cancel_token: &CancellationToken,
        stage: &str,
        document_id: &str,
    ) -> TaskResult<()> {
        if cancel_token.is_cancelled() {
            let post_graph = Self::is_post_graph_stage(stage);
            let (status, msg) = if post_graph {
                (
                    "partial_success",
                    format!(
                        "Processing interrupted during '{}' — graph data is already saved. \
                         Reprocess to finalize lineage.",
                        stage
                    ),
                )
            } else {
                (
                    "cancelled",
                    format!(
                        "Task cancelled during '{}' stage for document {}",
                        stage, document_id
                    ),
                )
            };
            tracing::info!(
                error.source = "task_processor",
                error.action = if post_graph { "interrupted_post_graph" } else { "cancelled" },
                document_id = %document_id,
                stage = %stage,
                error.message = %msg,
                "Task cancelled"
            );
            self.update_document_status(document_id, status, Some(&msg))
                .await
                .ok();
            return Err(TaskError::Cancelled(msg));
        }
        Ok(())
    }
}
