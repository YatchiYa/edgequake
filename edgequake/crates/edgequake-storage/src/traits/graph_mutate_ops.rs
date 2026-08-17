//! Graph mutation operations (SPEC-017 ISP Phase 2b).

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::Result;

<<<<<<< HEAD
=======
/// How conflict updates apply property maps on native AGE upsert.
///
/// - [`MergeSources`](Self::MergeSources): ingest-safe — `eq_merge_graph_properties`
///   unions `source_ids` / `source_chunk_ids` (SPEC-058).
/// - [`Replace`](Self::Replace): cascade prune — set `properties = EXCLUDED.properties`
///   so subtractive `source_ids` writes stick (SPEC-098 / LAW-098-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphPropertyWriteMode {
    /// Concurrent ingest: union source lineage arrays.
    #[default]
    MergeSources,
    /// Document cascade shared-entity prune: full property replace.
    Replace,
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
/// Upsert, delete, and clear graph data.
///
/// # Batch contract (P-G10 / RC-15, LSP)
///
/// `upsert_nodes_batch` and `upsert_edges_batch` are **required** (no default
/// impl). They MUST persist all items in a single storage round-trip (or one
/// logical transaction) — not loop over the per-item methods. This closes the
/// LSP trap where the memory adapter inherited an O(N) default and silently
/// made every "batch" call N round-trips. Callers can now rely on batch
/// performance semantics regardless of backend.
#[async_trait]
pub trait GraphStorageMutateOps: Send + Sync {
    async fn upsert_node(
        &self,
        node_id: &str,
        properties: HashMap<String, serde_json::Value>,
    ) -> Result<()>;

    /// Batch upsert all nodes in one storage operation (required; no default).
<<<<<<< HEAD
=======
    ///
    /// Equivalent to [`upsert_nodes_batch_with_mode`](Self::upsert_nodes_batch_with_mode)
    /// with [`GraphPropertyWriteMode::MergeSources`].
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    async fn upsert_nodes_batch(
        &self,
        nodes: &[(String, HashMap<String, serde_json::Value>)],
    ) -> Result<()>;

<<<<<<< HEAD
=======
    /// Batch upsert with explicit property write mode (SPEC-098 cascade prune).
    ///
    /// Default delegates to [`upsert_nodes_batch`](Self::upsert_nodes_batch)
    /// for both modes (memory already replaces; ingest callers keep MergeSources).
    async fn upsert_nodes_batch_with_mode(
        &self,
        nodes: &[(String, HashMap<String, serde_json::Value>)],
        mode: GraphPropertyWriteMode,
    ) -> Result<()> {
        let _ = mode;
        self.upsert_nodes_batch(nodes).await
    }

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    async fn delete_node(&self, node_id: &str) -> Result<()>;

    /// Batch-delete nodes (and incident edges). Default loops `delete_node`;
    /// Postgres native path is O(K log N) one round-trip (SPEC-060 compensate).
    async fn delete_nodes_batch(&self, node_ids: &[String]) -> Result<()> {
        for id in node_ids {
            self.delete_node(id).await?;
        }
        Ok(())
    }

    /// Delete a node only when its stored tenant/workspace match (defense in depth).
    ///
    /// Returns `Ok(true)` when a node was deleted, `Ok(false)` when no matching node
    /// exists (including cross-tenant IDOR attempts — caller should map to 404).
    async fn delete_node_scoped(
        &self,
        node_id: &str,
        tenant_id: &str,
        workspace_id: &str,
    ) -> Result<bool>;

    async fn upsert_edge(
        &self,
        source: &str,
        target: &str,
        properties: HashMap<String, serde_json::Value>,
    ) -> Result<()>;

    /// Batch upsert all edges in one storage operation (required; no default).
<<<<<<< HEAD
=======
    ///
    /// Equivalent to [`upsert_edges_batch_with_mode`](Self::upsert_edges_batch_with_mode)
    /// with [`GraphPropertyWriteMode::MergeSources`].
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    async fn upsert_edges_batch(
        &self,
        edges: &[(String, String, HashMap<String, serde_json::Value>)],
    ) -> Result<()>;

<<<<<<< HEAD
    async fn delete_edge(&self, source: &str, target: &str) -> Result<()>;

    /// Batch-delete edges by `(source, target)` pairs.
    ///
    /// Default loops `delete_edge`. Postgres native path is one `ANY` round-trip
    /// (SPEC-060 style) so document cascade delete stays O(1) storage ops.
    async fn delete_edges_batch(&self, edges: &[(String, String)]) -> Result<()> {
        for (source, target) in edges {
            self.delete_edge(source, target).await?;
=======
    /// Batch edge upsert with explicit property write mode (SPEC-098 cascade prune).
    async fn upsert_edges_batch_with_mode(
        &self,
        edges: &[(String, String, HashMap<String, serde_json::Value>)],
        mode: GraphPropertyWriteMode,
    ) -> Result<()> {
        let _ = mode;
        self.upsert_edges_batch(edges).await
    }

    async fn delete_edge(&self, source: &str, target: &str) -> Result<()>;

    /// Batch-delete edges by `(source, target, rel_type)` triples (SPEC-098 D-30).
    ///
    /// `rel_type` must be normalized (see [`crate::normalize_rel_type`]). Cascade
    /// exclusive prune deletes one multigraph sister at a time — never all rels
    /// between endpoints. Default loops `delete_edge` (all rels) only when callers
    /// still use the legacy pair API via adapters that expand triples.
    async fn delete_edges_batch(&self, edges: &[(String, String, String)]) -> Result<()> {
        // Fallback: collapse to endpoint pairs (may over-delete sisters). Adapters
        // that implement D-30 should override with precise SQL/memory deletes.
        let mut pairs: Vec<(String, String)> = edges
            .iter()
            .map(|(s, t, _)| (s.clone(), t.clone()))
            .collect();
        pairs.sort();
        pairs.dedup();
        for (source, target) in pairs {
            self.delete_edge(&source, &target).await?;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        }
        Ok(())
    }

    /// Delete an edge only when tenant/workspace properties match.
    async fn delete_edge_scoped(
        &self,
        source: &str,
        target: &str,
        tenant_id: &str,
        workspace_id: &str,
    ) -> Result<bool>;

    async fn clear(&self) -> Result<()>;

    async fn clear_workspace(&self, workspace_id: &uuid::Uuid) -> Result<(usize, usize)> {
        let _ = workspace_id;
        Ok((0, 0))
    }
}
