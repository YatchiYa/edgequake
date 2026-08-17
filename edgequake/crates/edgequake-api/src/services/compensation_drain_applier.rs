//! SPEC-091 IW3 — production compensation-quarantine drain applier (GAP-091-18).
//!
//! Retries failed saga cleanups recorded in `compensation_quarantine.payload`.
//! Never silently succeeds: unknown kinds and persistent failures surface as
//! drain errors and eventually dead-letter after bounded attempts.

use std::sync::Arc;

use edgequake_storage::compensation_drain::{
    spawn_compensation_drain, DrainConfig, QuarantineEntry,
};
use edgequake_storage::traits::{GraphStorage, KVStorage, VectorStorage};
use sqlx::PgPool;

/// Spawn the periodic drain with a real retract applier when mode is not `off`.
pub fn spawn_compensation_drain_applier(
    pool: PgPool,
    kv: Arc<dyn KVStorage>,
    vector: Arc<dyn VectorStorage>,
    graph: Arc<dyn GraphStorage>,
) -> Option<tokio::task::JoinHandle<()>> {
    let config = DrainConfig::from_env();
    spawn_compensation_drain(pool, config, move |entry| {
        let kv = Arc::clone(&kv);
        let vector = Arc::clone(&vector);
        let graph = Arc::clone(&graph);
        async move {
            apply_compensation_entry(kv.as_ref(), vector.as_ref(), graph.as_ref(), entry).await
        }
    })
}

async fn apply_compensation_entry(
    kv: &dyn KVStorage,
    vector: &dyn VectorStorage,
    graph: &dyn GraphStorage,
    entry: QuarantineEntry,
) -> Result<(), String> {
    let kind = entry
        .payload
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let artifact_id = entry
        .payload
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let doc_id = entry.document_id.to_string();

    tracing::info!(
        entry_id = %entry.entry_id,
        document_id = %doc_id,
        kind = %kind,
        attempt = entry.attempt_count,
        "compensation drain applying retract"
    );

    match kind {
        // SPEC-091 RM1: KV store dropped (mig 125) — ack without delete.
        "kv" => {
            let _ = (kv, artifact_id);
            Ok(())
        }
        "vector" => {
            let ids: Vec<String> = artifact_id
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            if ids.is_empty() {
                return Err("vector compensation entry has no ids".into());
            }
            vector
                .delete(&ids)
                .await
                .map_err(|e| format!("vector retract failed: {e}"))
        }
        "edge" => {
            let parts: Vec<&str> = artifact_id.split("->").collect();
            if parts.len() != 2 {
                return Err(format!("malformed edge id '{artifact_id}'"));
            }
            graph
                .delete_edge(parts[0], parts[1])
                .await
                .map_err(|e| format!("edge retract failed: {e}"))
        }
        "node" => {
            let nodes: Vec<String> = artifact_id
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            if nodes.is_empty() {
                return Err("node compensation entry has no node ids".into());
            }
            graph
                .delete_nodes_batch(&nodes)
                .await
                .map_err(|e| format!("node retract failed: {e}"))
        }
        other => Err(format!(
            "unknown compensation kind '{other}' for entry {} — manual reconciliation required",
            entry.entry_id
        )),
    }
}
