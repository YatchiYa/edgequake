//! Relationship merge, update, creation, and placeholder node logic.

use std::collections::HashMap;

use edgequake_storage::{EntityId, GraphEdge, GraphStorage, VectorStorage};

use crate::error::Result;
use crate::extractor::ExtractedRelationship;

use super::merge_progress::MergeProgressCtx;
use super::{merge_descriptions, merge_progress, metadata, MergeStats, RelationLineageLink};

impl<G: GraphStorage + ?Sized, V: VectorStorage + ?Sized> super::KnowledgeGraphMerger<G, V> {
    /// Collect batched relationship vector upserts (P-G4-merger).
    pub(super) fn collect_relationship_vector_batch(
        &self,
        relationships: &[ExtractedRelationship],
    ) -> Vec<(String, Vec<f32>, serde_json::Value)> {
        let mut batch = Vec::new();
        for rel in relationships {
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

        for chunk_start in (0..total_valid).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(total_valid);
            let mut lineage_batch: Vec<RelationLineageLink> = Vec::new();

            for (rel, source_key, target_key) in &valid[chunk_start..chunk_end] {
                if let Some(ref chunk_id) = rel.source_chunk_id {
                    lineage_batch.push(RelationLineageLink {
                        chunk_id: chunk_id.clone(),
                        source_entity: source_key.clone(),
                        target_entity: target_key.clone(),
                        workspace_id: ws.clone(),
                    });
                }

                if let Some(existing) = edge_map.get(&(source_key.clone(), target_key.clone())) {
                    let mut edge = existing.clone();
                    self.update_relationship_edge(&mut edge, rel).await?;
                    edge_batch.push((
                        edge.source.clone(),
                        edge.target.clone(),
                        edge.properties.clone(),
                    ));
                    stats.relationships_updated += 1;
                } else {
                    let edge = self.create_relationship_edge(source_key, target_key, rel)?;
                    if rel.embedding.is_some() {
                        let rel_id =
                            format!("{}->{}:{}", source_key, target_key, rel.relation_type);
                        stats.artifacts.relationship_vector_ids.push(rel_id);
                    }
                    stats
                        .artifacts
                        .graph_edges_created
                        .push((source_key.clone(), target_key.clone()));
                    edge_batch.push((edge.source, edge.target, edge.properties));
                    stats.relationships_created += 1;
                }
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

    /// Update an existing relationship edge.
    async fn update_relationship_edge(
        &self,
        edge: &mut GraphEdge,
        rel: &ExtractedRelationship,
    ) -> Result<()> {
        // Merge descriptions
        let existing_desc = edge
            .properties
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // ── SPEC-032 W-06: Similarity gate (parity with entity merge) ─────────
        let similarity = super::entity::description_similarity(existing_desc, &rel.description);
        let use_llm = self.config.use_llm_summarization
            && self.summarizer.is_some()
            && similarity < self.config.description_similarity_threshold;

        // Use LLM summarizer if available and enabled
        let merged_desc = if use_llm {
            if let Some(summarizer) = &self.summarizer {
                // Use LLM to intelligently merge relationship descriptions
                let descriptions = vec![existing_desc.to_string(), rel.description.clone()];
                match summarizer
                    .merge_relationship_descriptions(&rel.source, &rel.target, &descriptions)
                    .await
                {
                    Ok(merged) => {
                        tracing::debug!(
                            source = %rel.source,
                            target = %rel.target,
                            similarity,
                            "LLM relationship description merge completed"
                        );
                        merged
                    }
                    Err(e) => {
                        tracing::warn!(
                            source = %rel.source,
                            target = %rel.target,
                            error = %e,
                            "LLM summarization failed, falling back to simple merge"
                        );
                        merge_descriptions(
                            existing_desc,
                            &rel.description,
                            self.config.max_description_length,
                        )
                    }
                }
            } else {
                merge_descriptions(
                    existing_desc,
                    &rel.description,
                    self.config.max_description_length,
                )
            }
        } else {
            if similarity >= self.config.description_similarity_threshold {
                tracing::debug!(
                    source = %rel.source,
                    target = %rel.target,
                    similarity,
                    threshold = self.config.description_similarity_threshold,
                    "Similarity gate: skipping LLM summarizer (descriptions near-identical)"
                );
            }
            if rel.description.len() > existing_desc.len() {
                rel.description.clone()
            } else {
                existing_desc.to_string()
            }
        };

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

        Ok(())
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
        properties.insert(
            "relation_type".to_string(),
            serde_json::Value::String(rel.relation_type.clone()),
        );

        // Source tracking for citations (LightRAG parity)
        if let Some(ref chunk_id) = rel.source_chunk_id {
            properties.insert(
                "source_chunk_id".to_string(),
                serde_json::Value::String(chunk_id.clone()),
            );
        }
        if let Some(ref doc_id) = rel.source_document_id {
            properties.insert(
                "source_document_id".to_string(),
                serde_json::Value::String(doc_id.clone()),
            );
        }
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
