//! Relationship merge, update, creation, and placeholder node logic.
//!
//! # SPEC-047 P7b / P7d
//! - Parallel unique-edge description merges (`merge_max_async`)
//! - SOURCE_IDS KEEP: skip saturated description updates

use std::collections::HashMap;

use edgequake_storage::{EntityId, GraphEdge, GraphStorage, VectorStorage};
use futures::stream::{self, StreamExt};

use crate::error::Result;
use crate::extractor::ExtractedRelationship;

use super::merge_limits::{
    apply_source_ids_limit, merge_source_ids, should_skip_description_update_keep,
    source_chunk_ids_from_properties,
};
use super::merge_progress::{self, MergeProgressCtx};
use super::{metadata, MergeStats, RelationLineageLink};

/// One unique-edge merge result (P7b parallel collect).
struct RelMergeOutcome {
    source: String,
    target: String,
    properties: HashMap<String, serde_json::Value>,
    is_new: bool,
    skipped_saturated: bool,
    vector_id: Option<String>,
    /// Updated edge for map coherence (None when saturated skip).
    edge_for_map: Option<GraphEdge>,
}

impl<G: GraphStorage + ?Sized, V: VectorStorage + ?Sized> super::KnowledgeGraphMerger<G, V> {
    /// Collect batched relationship vector upserts (P-G4-merger).
    ///
    /// SPEC-047 P6: dedupe by `(src, tgt, type)` before upsert (O(unique)).
    pub(super) fn collect_relationship_vector_batch(
        &self,
        relationships: &[ExtractedRelationship],
    ) -> Vec<(String, Vec<f32>, serde_json::Value)> {
        let unique = crate::pipeline::helpers::unique_embed::dedupe_relationships_by_endpoints(
            relationships,
        );
        let mut batch = Vec::with_capacity(unique.len());
        for rel in &unique {
            let source_id = EntityId::new(&rel.source);
            let target_id = EntityId::new(&rel.target);
            let source_key = source_id.as_graph_node_id();
            let target_key = target_id.as_graph_node_id();
            if source_key.is_empty() || target_key.is_empty() || source_key == target_key {
                continue;
            }
            let Some(embedding) = rel.embedding.as_ref() else {
                continue;
            };
            let rel_id = format!("{}->{}:{}", source_key, target_key, rel.relation_type);
            let scope = metadata::TenantScope {
                tenant_id: &self.tenant_id,
                workspace_id: &self.workspace_id,
            };
            let metadata =
                metadata::relationship_vector_metadata(rel, source_key, target_key, scope);
            batch.push((rel_id, embedding.clone(), metadata));
        }
        batch
    }

    /// Merge relationships with batched graph reads/writes (P-G4-graph).
    ///
    /// SPEC-045: Emits incremental merge progress per chunk and batches lineage
    /// writes so the UI advances during long runs (fixes frozen 0/N counters).
    ///
    /// # Within-batch endpoint dedup (parity with entity W-03)
    ///
    /// AGE unique index `idx_edge_source_target_unique` is `(source_id, target_id)`
    /// only — not `relation_type`. Multiple extracted edges for the same pair
    /// (cross-chunk, multi-type) must collapse before `upsert_edges_batch`, or
    /// native `ON CONFLICT DO UPDATE` fails with "cannot affect row a second time".
    pub(super) async fn merge_relationships_batch(
        &self,
        relationships: Vec<ExtractedRelationship>,
        stats: &mut MergeStats,
        progress: Option<MergeProgressCtx<'_>>,
    ) -> Result<()> {
        if relationships.is_empty() {
            return Ok(());
        }

        let mut valid = Vec::new();
        let mut endpoint_keys = Vec::new();

        for rel in relationships {
            let source_id = EntityId::new(&rel.source);
            let target_id = EntityId::new(&rel.target);
            let source_key = source_id.as_graph_node_id().to_string();
            let target_key = target_id.as_graph_node_id().to_string();

            if source_key == target_key {
                tracing::debug!(
                    source = %source_key,
                    "Merger: skipping self-referencing relationship (BR0006)"
                );
                continue;
            }
            if source_key.is_empty() || target_key.is_empty() {
                tracing::debug!(
                    raw_source = %rel.source,
                    raw_target = %rel.target,
                    "Merger: skipping relationship with empty normalized endpoint"
                );
                continue;
            }

            if !endpoint_keys.contains(&source_key) {
                endpoint_keys.push(source_key.clone());
            }
            if !endpoint_keys.contains(&target_key) {
                endpoint_keys.push(target_key.clone());
            }
            valid.push((rel, source_key, target_key));
        }

        if valid.is_empty() {
            return Ok(());
        }

        // Domain dedup: one ExtractedRelationship per (source_key, target_key).
        // Last-write-wins on relation_type/description; merge keywords + weight.
        let valid = dedupe_relationships_by_endpoints(valid);

        let total_valid = valid.len();
        if let Some(ctx) = progress {
            ctx.emit_relationship_graph(
                0,
                stats.relationships_created,
                stats.relationships_updated,
            );
        }

        let existing_nodes = self.graph_storage.get_nodes_batch(&endpoint_keys).await?;
        let incident_edges = self
            .graph_storage
            .get_edges_for_nodes_batch(&endpoint_keys)
            .await?;
        let mut edge_map: HashMap<(String, String), GraphEdge> = HashMap::new();
        for edge in incident_edges {
            edge_map.insert((edge.source.clone(), edge.target.clone()), edge);
        }

        let mut placeholder_batch: Vec<(String, HashMap<String, serde_json::Value>)> = Vec::new();
        let mut placeholders: HashMap<String, String> = HashMap::new();
        for (rel, source_key, target_key) in &valid {
            if !existing_nodes.contains_key(source_key) {
                placeholders
                    .entry(source_key.clone())
                    .or_insert_with(|| rel.source.clone());
            }
            if !existing_nodes.contains_key(target_key) {
                placeholders
                    .entry(target_key.clone())
                    .or_insert_with(|| rel.target.clone());
            }
        }
        for (key, label) in placeholders {
            placeholder_batch.push((key.clone(), self.placeholder_node_properties(&label)));
            stats.artifacts.graph_nodes_created.push(key);
        }

        if !placeholder_batch.is_empty() {
            self.graph_storage
                .upsert_nodes_batch(&placeholder_batch)
                .await?;
        }

        let mut edge_batch: Vec<(String, String, HashMap<String, serde_json::Value>)> =
            Vec::with_capacity(total_valid);

        let ws = self
            .workspace_id
            .as_deref()
            .unwrap_or("default")
            .to_string();
        let chunk_size = merge_progress::relationship_merge_chunk_size();
        let concurrency = self.config.merge_max_async.max(1);

        for chunk_start in (0..total_valid).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(total_valid);
            let mut lineage_batch: Vec<RelationLineageLink> = Vec::new();

            let works: Vec<(
                usize,
                ExtractedRelationship,
                String,
                String,
                Option<GraphEdge>,
            )> = valid[chunk_start..chunk_end]
                .iter()
                .enumerate()
                .map(|(offset, (rel, source_key, target_key))| {
                    let pair = (source_key.clone(), target_key.clone());
                    let existing = edge_map.get(&pair).cloned();
                    (
                        chunk_start + offset,
                        rel.clone(),
                        source_key.clone(),
                        target_key.clone(),
                        existing,
                    )
                })
                .collect();

            for (rel, source_key, target_key) in &valid[chunk_start..chunk_end] {
                if let Some(ref chunk_id) = rel.source_chunk_id {
                    lineage_batch.push(RelationLineageLink {
                        chunk_id: chunk_id.clone(),
                        source_entity: source_key.clone(),
                        target_entity: target_key.clone(),
                        workspace_id: ws.clone(),
                    });
                }
            }

            // SPEC-047 P7b: parallel unique-edge description merges within chunk.
            let outcomes: Vec<Result<(usize, RelMergeOutcome)>> = stream::iter(works)
                .map(|(idx, rel, source_key, target_key, existing)| async move {
                    let outcome = self
                        .build_relationship_merge_outcome(&rel, &source_key, &target_key, existing)
                        .await?;
                    Ok((idx, outcome))
                })
                .buffer_unordered(concurrency)
                .collect()
                .await;

            let mut ordered: Vec<(usize, RelMergeOutcome)> = Vec::with_capacity(outcomes.len());
            for item in outcomes {
                match item {
                    Ok(pair) => ordered.push(pair),
                    Err(e) => {
                        stats.errors += 1;
                        tracing::warn!(
                            error.source = "pipeline_merger",
                            error.action = "merge_relationship",
                            error.message = %e,
                            "Failed to merge relationship"
                        );
                    }
                }
            }
            ordered.sort_by_key(|(i, _)| *i);

            for (_idx, outcome) in ordered {
                if outcome.skipped_saturated {
                    stats.relationships_skipped_saturated += 1;
                    continue;
                }
                // Relationship vector IDs are recorded in upsert_vectors_chunked (SPEC-057 P3).
                let _ = outcome.vector_id;
                if outcome.is_new {
                    stats
                        .artifacts
                        .graph_edges_created
                        .push((outcome.source.clone(), outcome.target.clone()));
                    stats.relationships_created += 1;
                } else {
                    stats.relationships_updated += 1;
                }
                if let Some(edge) = outcome.edge_for_map {
                    edge_map.insert((edge.source.clone(), edge.target.clone()), edge);
                }
                edge_batch.push((outcome.source, outcome.target, outcome.properties));
            }

            if !lineage_batch.is_empty() {
                self.lineage_sink
                    .record_relation_links_batch(&lineage_batch)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(
                            count = lineage_batch.len(),
                            error = %e,
                            "Lineage sink record_relation_links_batch failed (best-effort)"
                        );
                    });
            }

            if let Some(ctx) = progress {
                ctx.emit_relationship_graph(
                    chunk_end,
                    stats.relationships_created,
                    stats.relationships_updated,
                );
            }
        }

        // Storage-layer dedupe is the SSOT for ON CONFLICT safety; still collapse
        // here so create/update stats and artifacts stay aligned with one edge/pair.
        let edge_batch = edgequake_storage::dedupe_edges_by_endpoints(&edge_batch);

        if !edge_batch.is_empty() {
            if let Some(ctx) = progress {
                ctx.emit_relationship_graph(
                    total_valid,
                    stats.relationships_created,
                    stats.relationships_updated,
                );
            }
            self.graph_storage.upsert_edges_batch(&edge_batch).await?;
        }

        if let Some(ctx) = progress {
            ctx.emit_relationship_graph(
                total_valid,
                stats.relationships_created,
                stats.relationships_updated,
            );
        }

        Ok(())
    }

    fn placeholder_node_properties(&self, label: &str) -> HashMap<String, serde_json::Value> {
        let mut properties = HashMap::new();
        properties.insert(
            "entity_type".to_string(),
            serde_json::Value::String("UNKNOWN".to_string()),
        );
        properties.insert(
            "description".to_string(),
            serde_json::Value::String(String::new()),
        );
        properties.insert(
            "label".to_string(),
            serde_json::Value::String(label.to_string()),
        );
        if let Some(tenant_id) = &self.tenant_id {
            properties.insert(
                "tenant_id".to_string(),
                serde_json::Value::String(tenant_id.clone()),
            );
        }
        if let Some(workspace_id) = &self.workspace_id {
            properties.insert(
                "workspace_id".to_string(),
                serde_json::Value::String(workspace_id.clone()),
            );
        }
        properties
    }

    async fn build_relationship_merge_outcome(
        &self,
        rel: &ExtractedRelationship,
        source_key: &str,
        target_key: &str,
        existing: Option<GraphEdge>,
    ) -> Result<RelMergeOutcome> {
        if let Some(mut edge) = existing {
            let mutated = self.update_relationship_edge(&mut edge, rel).await?;
            if !mutated {
                return Ok(RelMergeOutcome {
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    properties: edge.properties,
                    is_new: false,
                    skipped_saturated: true,
                    vector_id: None,
                    edge_for_map: None,
                });
            }
            return Ok(RelMergeOutcome {
                source: edge.source.clone(),
                target: edge.target.clone(),
                properties: edge.properties.clone(),
                is_new: false,
                skipped_saturated: false,
                vector_id: None,
                edge_for_map: Some(edge),
            });
        }

        let edge = self.create_relationship_edge(source_key, target_key, rel)?;
        let vector_id = rel
            .embedding
            .as_ref()
            .map(|_| format!("{}->{}:{}", source_key, target_key, rel.relation_type));
        Ok(RelMergeOutcome {
            source: edge.source.clone(),
            target: edge.target.clone(),
            properties: edge.properties.clone(),
            is_new: true,
            skipped_saturated: false,
            vector_id,
            edge_for_map: Some(edge),
        })
    }

    /// Update an existing relationship edge (SPEC-047 P7a fragment gate + P7d KEEP).
    ///
    /// Returns `false` when KEEP-saturated → caller skips upsert.
    async fn update_relationship_edge(
        &self,
        edge: &mut GraphEdge,
        rel: &ExtractedRelationship,
    ) -> Result<bool> {
        let incoming_ids: Vec<String> = rel
            .source_chunk_id
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect();
        let existing_chunk_ids = source_chunk_ids_from_properties(&edge.properties);
        if should_skip_description_update_keep(
            &existing_chunk_ids,
            &incoming_ids,
            self.config.max_source_ids_per_relation,
            self.config.source_ids_limit_method,
        ) {
            tracing::debug!(
                source = %rel.source,
                target = %rel.target,
                existing = existing_chunk_ids.len(),
                max = self.config.max_source_ids_per_relation,
                "P7d KEEP: skip relationship description update (saturated)"
            );
            return Ok(false);
        }

        let existing_desc = edge
            .properties
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let merged_desc = self
            .resolve_relationship_description(
                &rel.source,
                &rel.target,
                existing_desc,
                &rel.description,
            )
            .await?;

        edge.properties.insert(
            "description".to_string(),
            serde_json::Value::String(merged_desc),
        );

        // Update weight (use weighted average)
        let existing_weight = edge
            .properties
            .get("weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5) as f32;

        let new_weight = (existing_weight + rel.weight) / 2.0;
        edge.properties.insert(
            "weight".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(new_weight as f64).unwrap()),
        );

        // Prefer newer relation_type when present (last-write-wins for type label).
        if !rel.relation_type.is_empty() {
            edge.properties.insert(
                "relation_type".to_string(),
                serde_json::Value::String(rel.relation_type.clone()),
            );
        }

        // Merge keywords
        let mut keywords: Vec<String> = edge
            .properties
            .get("keywords")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        for keyword in &rel.keywords {
            if !keywords.contains(keyword) {
                keywords.push(keyword.clone());
            }
        }

        // BR0004: Relationship keywords max 5 per edge
        // WHY: Excessive keywords dilute semantic relevance and inflate storage.
        // Keep the first 5 (oldest = most established context).
        keywords.truncate(5);

        edge.properties
            .insert("keywords".to_string(), serde_json::json!(keywords));

        // Merge + cap source chunk IDs (P7d)
        let merged_ids = merge_source_ids(&existing_chunk_ids, &incoming_ids);
        let capped = apply_source_ids_limit(
            &merged_ids,
            self.config.max_source_ids_per_relation,
            self.config.source_ids_limit_method,
        );
        super::lineage::insert_chunk_lineage_properties(&mut edge.properties, &capped);
        if let Some(first) = capped.first() {
            edge.properties.insert(
                "source_chunk_id".to_string(),
                serde_json::Value::String(first.clone()),
            );
        }
        super::lineage::merge_and_insert_document_lineage(
            &mut edge.properties,
            rel.source_document_id.as_deref(),
            &capped,
        );

        Ok(true)
    }

    /// Create a new relationship edge.
    fn create_relationship_edge(
        &self,
        source_key: &str,
        target_key: &str,
        rel: &ExtractedRelationship,
    ) -> Result<GraphEdge> {
        let mut properties = HashMap::new();
        properties.insert(
            "relation_type".to_string(),
            serde_json::Value::String(rel.relation_type.clone()),
        );
        properties.insert(
            "description".to_string(),
            serde_json::Value::String(rel.description.clone()),
        );
        properties.insert(
            "weight".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(rel.weight as f64).unwrap()),
        );
        properties.insert("keywords".to_string(), serde_json::json!(rel.keywords));

        // Source tracking for citations (LightRAG parity)
        let incoming: Vec<String> = rel
            .source_chunk_id
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect();
        let capped = apply_source_ids_limit(
            &incoming,
            self.config.max_source_ids_per_relation,
            self.config.source_ids_limit_method,
        );
        super::lineage::insert_chunk_lineage_properties(&mut properties, &capped);
        if let Some(first) = capped.first() {
            properties.insert(
                "source_chunk_id".to_string(),
                serde_json::Value::String(first.clone()),
            );
        }
        super::lineage::merge_and_insert_document_lineage(
            &mut properties,
            rel.source_document_id.as_deref(),
            &capped,
        );
        if let Some(ref file_path) = rel.source_file_path {
            properties.insert(
                "source_file_path".to_string(),
                serde_json::Value::String(file_path.clone()),
            );
        }

        // Add tenant context
        if let Some(tenant_id) = &self.tenant_id {
            properties.insert(
                "tenant_id".to_string(),
                serde_json::Value::String(tenant_id.clone()),
            );
        }
        if let Some(workspace_id) = &self.workspace_id {
            properties.insert(
                "workspace_id".to_string(),
                serde_json::Value::String(workspace_id.clone()),
            );
        }

        Ok(GraphEdge {
            source: source_key.to_string(),
            target: target_key.to_string(),
            properties,
        })
    }
}

/// Collapse extracted relationships to one row per `(source_key, target_key)`.
///
/// # Policy (First Principles / AGE unique index)
///
/// - Last-write-wins for `relation_type`, `description`, embeddings, source ids
/// - Keywords union (capped later at write time)
/// - Weight = max of seen weights (preserve strongest signal)
fn dedupe_relationships_by_endpoints(
    rows: Vec<(ExtractedRelationship, String, String)>,
) -> Vec<(ExtractedRelationship, String, String)> {
    let mut order: Vec<(String, String)> = Vec::new();
    let mut map: HashMap<(String, String), ExtractedRelationship> = HashMap::new();

    for (rel, source_key, target_key) in rows {
        let key = (source_key.clone(), target_key.clone());
        if let Some(existing) = map.get_mut(&key) {
            if rel.description.len() > existing.description.len() {
                existing.description = rel.description.clone();
            }
            if !rel.relation_type.is_empty() {
                existing.relation_type = rel.relation_type.clone();
            }
            if rel.weight > existing.weight {
                existing.weight = rel.weight;
            }
            for kw in &rel.keywords {
                if !existing.keywords.contains(kw) {
                    existing.keywords.push(kw.clone());
                }
            }
            if rel.embedding.is_some() {
                existing.embedding = rel.embedding.clone();
            }
            if rel.source_chunk_id.is_some() {
                existing.source_chunk_id = rel.source_chunk_id.clone();
            }
            if rel.source_document_id.is_some() {
                existing.source_document_id = rel.source_document_id.clone();
            }
            if rel.source_file_path.is_some() {
                existing.source_file_path = rel.source_file_path.clone();
            }
        } else {
            order.push(key.clone());
            map.insert(key, rel);
        }
    }

    order
        .into_iter()
        .filter_map(|(src, tgt)| {
            map.remove(&(src.clone(), tgt.clone()))
                .map(|rel| (rel, src, tgt))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_relationships_collapses_same_endpoints() {
        let rows = vec![
            (
                ExtractedRelationship::new("Alice", "Bob", "KNOWS").with_description("a"),
                "ALICE".into(),
                "BOB".into(),
            ),
            (
                ExtractedRelationship::new("Alice", "Bob", "WORKS_WITH")
                    .with_description("longer description")
                    .with_weight(0.9)
                    .with_keywords(vec!["x".into()]),
                "ALICE".into(),
                "BOB".into(),
            ),
            (
                ExtractedRelationship::new("Carol", "Dave", "RELATED"),
                "CAROL".into(),
                "DAVE".into(),
            ),
        ];
        let out = dedupe_relationships_by_endpoints(rows);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].1, "ALICE");
        assert_eq!(out[0].2, "BOB");
        assert_eq!(out[0].0.relation_type, "WORKS_WITH");
        assert_eq!(out[0].0.description, "longer description");
        assert!((out[0].0.weight - 0.9).abs() < f32::EPSILON);
        assert_eq!(out[0].0.keywords, vec!["x".to_string()]);
        assert_eq!(out[1].1, "CAROL");
    }
}
