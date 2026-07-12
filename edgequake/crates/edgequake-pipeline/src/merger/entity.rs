//! Entity merge, update, and creation logic for the knowledge graph.
//!
//! # SPEC-032 changes
//! - W-03: `merge_entities_batch` deduplicates within-document before graph read
//! - W-06: Similarity gate skips LLM summarizer when descriptions are near-identical
//!
//! # SPEC-047 P7b / P7d
//! - Parallel unique-entity description merges (`merge_max_async`)
//! - SOURCE_IDS KEEP: skip saturated description updates

use std::collections::HashMap;

use edgequake_storage::{EntityId, GraphNode, GraphStorage, VectorStorage};
use futures::stream::{self, StreamExt};

use crate::error::Result;
use crate::extractor::{ExtractedEntity, ExtractionResult};

use super::merge_limits::{
    apply_source_ids_limit, merge_source_ids, should_skip_description_update_keep,
    source_chunk_ids_from_properties,
};
use super::{metadata, MergeStats};

/// Outcome of one unique-entity merge (P7b parallel collect).
struct EntityMergeOutcome {
    node_id: String,
    properties: HashMap<String, serde_json::Value>,
    is_new: bool,
    skipped_saturated: bool,
    vector_id: Option<String>,
    graph_key_created: Option<String>,
}

/// Jaccard word-overlap similarity between two strings (pub for tests and external use).
pub fn description_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        // Identical strings: if both empty → 0 (no overlap to measure); if same non-empty → 1.0
        return if a.is_empty() { 0.0 } else { 1.0 };
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_words: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let b_words: std::collections::HashSet<&str> = b.split_whitespace().collect();
    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();
    if union == 0 {
        1.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Threshold above which we skip LLM summarization (descriptions are near-identical).
/// Tunable via `MergerConfig.description_similarity_threshold`.
/// Default exposed here for tests; runtime value comes from `MergerConfig`.
#[cfg(test)]
#[allow(dead_code)]
pub(crate) const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.85;

impl<G: GraphStorage + ?Sized, V: VectorStorage + ?Sized> super::KnowledgeGraphMerger<G, V> {
    /// Collect batched entity vector upserts for all extractions (P-G4-merger).
    ///
    /// SPEC-047 P6: dedupe by `EntityId` before upsert so HNSW pays O(unique),
    /// not O(mentions). Embeddings are already unique-before-embed; this is the
    /// storage-side half of the same first principle.
    pub(super) fn collect_entity_vector_batch(
        &self,
        results: &[ExtractionResult],
    ) -> Vec<(String, Vec<f32>, serde_json::Value)> {
        let all: Vec<_> = results
            .iter()
            .flat_map(|r| r.entities.iter().cloned())
            .collect();
        let unique = crate::pipeline::helpers::unique_embed::dedupe_entities_by_id(&all);
        let mut batch = Vec::with_capacity(unique.len());
        for entity in &unique {
            let entity_id = EntityId::new(&entity.name);
            if entity_id.is_empty() {
                continue;
            }
            let Some(embedding) = entity.embedding.as_ref() else {
                continue;
            };
            let scope = metadata::TenantScope {
                tenant_id: &self.tenant_id,
                workspace_id: &self.workspace_id,
            };
            let metadata = metadata::entity_vector_metadata(entity, &entity_id, scope);
            batch.push((entity_id.as_vector_id(), embedding.clone(), metadata));
        }
        batch
    }

    /// Merge entities with one `get_nodes_batch` + one `upsert_nodes_batch` (P-G4-graph).
    ///
    /// # SPEC-032 W-03: Within-document deduplication
    ///
    /// When the same entity appears in multiple chunks of the same document,
    /// the previous per-chunk loop would issue N get+upsert pairs.
    /// Now we deduplicate first: same-named entities within one call are merged
    /// in-memory (concatenating source_chunk_ids, taking the longer description)
    /// before a single get_nodes_batch reads the existing graph state.
    ///
    /// Edge case: entity with same name but different types → keep first type,
    /// append description (LightRAG convention: type is stable once set).
    pub(super) async fn merge_entities_batch(
        &self,
        entities: Vec<ExtractedEntity>,
        stats: &mut MergeStats,
    ) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        // ── Within-batch deduplication (SPEC-032 W-03) ───────────────────
        // If the same entity name appears in multiple chunks of this document,
        // merge them in-memory before hitting the database.
        // Use Vec<(key, entity)> to preserve first-seen order for determinism.
        let mut dedup_keys: Vec<String> = Vec::new();
        let mut dedup_map: HashMap<String, ExtractedEntity> = HashMap::new();

        for entity in entities {
            let entity_id = EntityId::new(&entity.name);
            if entity_id.is_empty() {
                tracing::warn!(
                    raw_name = %entity.name,
                    "Skipping entity with empty normalized name"
                );
                continue;
            }
            let key = entity_id.as_graph_node_id().to_string();
            if let Some(existing) = dedup_map.get_mut(&key) {
                // Merge descriptions: keep longer (richer)
                if entity.description.len() > existing.description.len() {
                    existing.description = entity.description.clone();
                }
                // Accumulate source chunks
                for cid in &entity.source_chunk_ids {
                    if !existing.source_chunk_ids.contains(cid) {
                        existing.source_chunk_ids.push(cid.clone());
                    }
                }
                // Merge source spans
                for span in &entity.source_spans {
                    if !existing.source_spans.contains(span) {
                        existing.source_spans.push(span.clone());
                    }
                }
                // Take max importance
                if entity.importance > existing.importance {
                    existing.importance = entity.importance;
                }
            } else {
                dedup_keys.push(key.clone());
                dedup_map.insert(key, entity);
            }
        }

        // Collect in insertion order (deterministic)
        let (keys, valid): (Vec<String>, Vec<ExtractedEntity>) = dedup_keys
            .into_iter()
            .filter_map(|k| dedup_map.remove(&k).map(|e| (k, e)))
            .unzip();

        if valid.is_empty() {
            return Ok(());
        }

        // Store entity types for relational sink (borrow before move into loop)
        let entity_types: Vec<String> = valid.iter().map(|e| e.entity_type.clone()).collect();
        let descriptions: Vec<String> = valid.iter().map(|e| e.description.clone()).collect();
        let source_chunk_ids: Vec<Vec<String>> =
            valid.iter().map(|e| e.source_chunk_ids.clone()).collect();

        let existing_map = self.graph_storage.get_nodes_batch(&keys).await?;
        let concurrency = self.config.merge_max_async.max(1);

        // SPEC-047 P7b: resolve unique entity merges concurrently (LLM-bound).
        let indexed: Vec<(usize, ExtractedEntity, String)> = valid
            .into_iter()
            .zip(keys.iter().cloned())
            .enumerate()
            .map(|(i, (entity, key))| (i, entity, key))
            .collect();

        let outcomes: Vec<Result<(usize, EntityMergeOutcome)>> = stream::iter(indexed)
            .map(|(i, entity, key)| {
                let existing = existing_map.get(&key).cloned();
                async move {
                    let outcome = self
                        .build_entity_merge_outcome(&entity, existing.as_ref())
                        .await?;
                    Ok((i, outcome))
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        let mut ordered: Vec<(usize, EntityMergeOutcome)> = Vec::with_capacity(outcomes.len());
        for item in outcomes {
            match item {
                Ok(pair) => ordered.push(pair),
                Err(e) => {
                    stats.errors += 1;
                    tracing::warn!(
                        error.source = "pipeline_merger",
                        error.action = "merge_entity",
                        error.message = %e,
                        "Failed to merge entity"
                    );
                }
            }
        }
        ordered.sort_by_key(|(i, _)| *i);

        let mut node_batch: Vec<(String, HashMap<String, serde_json::Value>)> =
            Vec::with_capacity(ordered.len());

        for (i, outcome) in ordered {
            if outcome.skipped_saturated {
                stats.entities_skipped_saturated += 1;
                continue;
            }
            if let Some(vid) = outcome.vector_id {
                stats.artifacts.entity_vector_ids.push(vid);
            }
            if let Some(key) = outcome.graph_key_created {
                stats.artifacts.graph_nodes_created.push(key);
            }
            node_batch.push((outcome.node_id, outcome.properties));
            if outcome.is_new {
                stats.entities_created += 1;
            } else {
                stats.entities_updated += 1;
            }

            let key = &keys[i];
            // CQRS relational sink (SPEC-021)
            self.relational_sink
                .upsert_entity(
                    key,
                    &entity_types[i],
                    &descriptions[i],
                    self.tenant_id.as_deref(),
                    self.workspace_id.as_deref(),
                    &source_chunk_ids[i],
                )
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        entity = %key,
                        error = %e,
                        "Relational entity sink failed (best-effort; graph write succeeded)"
                    );
                });
            // Lineage sink (SPEC-032 W-08 / SPEC-047 P4): batch chunk→entity links
            let ws = self.workspace_id.as_deref().unwrap_or("default");
            let links: Vec<crate::merger::EntityLineageLink> = source_chunk_ids[i]
                .iter()
                .map(|chunk_id| crate::merger::EntityLineageLink {
                    chunk_id: chunk_id.clone(),
                    entity_name: key.to_string(),
                    workspace_id: ws.to_string(),
                })
                .collect();
            if !links.is_empty() {
                self.lineage_sink
                    .record_entity_links_batch(&links)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(
                            entity = %key,
                            count = links.len(),
                            error = %e,
                            "Lineage sink record_entity_links_batch failed (best-effort)"
                        );
                    });
            }
        }

        if !node_batch.is_empty() {
            self.graph_storage.upsert_nodes_batch(&node_batch).await?;
        }

        Ok(())
    }

    async fn build_entity_merge_outcome(
        &self,
        entity: &ExtractedEntity,
        existing: Option<&GraphNode>,
    ) -> Result<EntityMergeOutcome> {
        let entity_id = EntityId::new(&entity.name);
        let entity_key = entity_id.as_graph_node_id().to_string();

        match existing.cloned() {
            Some(mut node) => {
                let mutated = self.update_entity_node(&mut node, entity).await?;
                if !mutated {
                    return Ok(EntityMergeOutcome {
                        node_id: node.id.clone(),
                        properties: node.properties,
                        is_new: false,
                        skipped_saturated: true,
                        vector_id: None,
                        graph_key_created: None,
                    });
                }
                Ok(EntityMergeOutcome {
                    node_id: node.id.clone(),
                    properties: node.properties,
                    is_new: false,
                    skipped_saturated: false,
                    vector_id: None,
                    graph_key_created: None,
                })
            }
            None => {
                let node = self.create_entity_node(entity)?;
                let vector_id = entity.embedding.as_ref().map(|_| entity_id.as_vector_id());
                Ok(EntityMergeOutcome {
                    node_id: node.id,
                    properties: node.properties,
                    is_new: true,
                    skipped_saturated: false,
                    vector_id,
                    graph_key_created: Some(entity_key),
                })
            }
        }
    }

    /// Update an existing entity node with new information.
    ///
    /// Returns `false` when SPEC-047 P7d KEEP saturated → caller skips upsert.
    ///
    /// # SPEC-047 P7a + SPEC-032 W-06
    ///
    /// Description merge is delegated to [`super::decide_description_merge`]
    /// (LightRAG fragment gate + Jaccard soft-resume skip). LLM runs only when
    /// the policy returns [`super::DescriptionMergeDecision::NeedsLlm`].
    async fn update_entity_node(
        &self,
        node: &mut GraphNode,
        entity: &ExtractedEntity,
    ) -> Result<bool> {
        let existing_chunk_ids = source_chunk_ids_from_properties(&node.properties);
        if should_skip_description_update_keep(
            &existing_chunk_ids,
            &entity.source_chunk_ids,
            self.config.max_source_ids_per_entity,
            self.config.source_ids_limit_method,
        ) {
            tracing::debug!(
                entity = %entity.name,
                existing = existing_chunk_ids.len(),
                max = self.config.max_source_ids_per_entity,
                "P7d KEEP: skip entity description update (saturated)"
            );
            return Ok(false);
        }

        let existing_desc = node
            .properties
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let merged_desc = self
            .resolve_entity_description(&entity.name, existing_desc, &entity.description)
            .await?;

        node.properties.insert(
            "description".to_string(),
            serde_json::Value::String(merged_desc),
        );

        // Update importance (take max)
        let existing_importance = node
            .properties
            .get("importance")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5) as f32;

        let new_importance = existing_importance.max(entity.importance);
        node.properties.insert(
            "importance".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(new_importance as f64).unwrap()),
        );

        // Merge source spans
        let mut sources: Vec<String> = node
            .properties
            .get("sources")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        for span in &entity.source_spans {
            if !sources.contains(span) && sources.len() < self.config.max_sources {
                sources.push(span.clone());
            }
        }

        node.properties
            .insert("sources".to_string(), serde_json::json!(sources));

        // Merge + cap source chunk IDs (P7d KEEP/FIFO)
        let merged_ids = merge_source_ids(&existing_chunk_ids, &entity.source_chunk_ids);
        let source_chunk_ids = apply_source_ids_limit(
            &merged_ids,
            self.config.max_source_ids_per_entity,
            self.config.source_ids_limit_method,
        );
        super::lineage::insert_chunk_lineage_properties(&mut node.properties, &source_chunk_ids);
        super::lineage::merge_and_insert_document_lineage(
            &mut node.properties,
            entity.source_document_id.as_deref(),
            &source_chunk_ids,
        );

        // Update source file path if not already set
        if !node.properties.contains_key("source_file_path") {
            if let Some(ref file_path) = entity.source_file_path {
                node.properties.insert(
                    "source_file_path".to_string(),
                    serde_json::Value::String(file_path.clone()),
                );
            }
        }

        Ok(true)
    }

    /// Create a new entity node.
    fn create_entity_node(&self, entity: &ExtractedEntity) -> Result<GraphNode> {
        let entity_id = EntityId::new(&entity.name);
        let entity_key = entity_id.as_graph_node_id().to_string();

        let mut properties = HashMap::new();
        properties.insert(
            "entity_type".to_string(),
            serde_json::Value::String(entity.entity_type.clone()),
        );
        properties.insert(
            "description".to_string(),
            serde_json::Value::String(entity.description.clone()),
        );
        properties.insert(
            "importance".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(entity.importance as f64).unwrap(),
            ),
        );
        properties.insert(
            "sources".to_string(),
            serde_json::json!(entity.source_spans),
        );
        properties.insert(
            "label".to_string(),
            serde_json::Value::String(entity.name.clone()),
        );

        // Source tracking for citations (LightRAG parity) + analytics reconcile
        let capped = apply_source_ids_limit(
            &entity.source_chunk_ids,
            self.config.max_source_ids_per_entity,
            self.config.source_ids_limit_method,
        );
        super::lineage::insert_chunk_lineage_properties(&mut properties, &capped);
        super::lineage::merge_and_insert_document_lineage(
            &mut properties,
            entity.source_document_id.as_deref(),
            &capped,
        );
        if let Some(ref file_path) = entity.source_file_path {
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

        Ok(GraphNode {
            id: entity_key,
            properties,
        })
    }
}
