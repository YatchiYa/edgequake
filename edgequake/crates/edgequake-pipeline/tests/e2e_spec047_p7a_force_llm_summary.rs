//! SPEC-047 P7a e2e: merger applies fragment gate; LLM backend called only at threshold.
//!
//! Uses an in-memory graph + counting [`DescriptionMergeBackend`] (SOLID DI).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use edgequake_pipeline::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};
use edgequake_pipeline::{
    DescriptionMergeBackend, KnowledgeGraphMerger, MergerConfig, Result as PipelineResult,
    GRAPH_FIELD_SEP,
};
use edgequake_storage::{
    GraphStorage, GraphStorageReadOps, MemoryGraphStorage, MemoryVectorStorage, VectorStorage,
};

const EMBED_DIM: usize = 8;

struct CountingMergeBackend {
    entity_calls: AtomicUsize,
    rel_calls: AtomicUsize,
}

impl CountingMergeBackend {
    fn new() -> Self {
        Self {
            entity_calls: AtomicUsize::new(0),
            rel_calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl DescriptionMergeBackend for CountingMergeBackend {
    async fn merge_entity_descriptions(
        &self,
        _entity_name: &str,
        descriptions: &[String],
    ) -> PipelineResult<String> {
        self.entity_calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("LLM_ENTITY({})", descriptions.len()))
    }

    async fn merge_relationship_descriptions(
        &self,
        _source: &str,
        _target: &str,
        descriptions: &[String],
    ) -> PipelineResult<String> {
        self.rel_calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("LLM_REL({})", descriptions.len()))
    }
}

fn entity(name: &str, desc: &str, chunk: &str) -> ExtractedEntity {
    ExtractedEntity::new(name, "CONCEPT", desc)
        .with_source_chunk_id(chunk)
        .with_importance(0.5)
}

fn rel(src: &str, tgt: &str, desc: &str, chunk: &str) -> ExtractedRelationship {
    ExtractedRelationship::new(src, tgt, "RELATED_TO")
        .with_description(desc)
        .with_source_chunk_id(chunk)
}

async fn setup_merger(
    force: usize,
    backend: Arc<CountingMergeBackend>,
) -> (
    KnowledgeGraphMerger<MemoryGraphStorage, MemoryVectorStorage>,
    Arc<MemoryGraphStorage>,
) {
    let graph = Arc::new(MemoryGraphStorage::new("p7a-e2e"));
    let vector = Arc::new(MemoryVectorStorage::new("p7a-e2e", EMBED_DIM));
    graph.initialize().await.unwrap();
    vector.initialize().await.unwrap();

    let config = MergerConfig {
        use_llm_summarization: true,
        force_llm_summary_on_merge: force,
        summary_max_tokens: 12_000,
        // Disable Jaccard skip so distinct short facts always accumulate.
        description_similarity_threshold: 1.01,
        ..MergerConfig::default()
    };

    let merger =
        KnowledgeGraphMerger::new(config, graph.clone(), vector).with_merge_backend(backend);
    (merger, graph)
}

#[tokio::test]
async fn e2e_entity_merge_joins_below_eight_without_llm() {
    let backend = Arc::new(CountingMergeBackend::new());
    let (merger, graph) = setup_merger(8, backend.clone()).await;

    let mut r0 = ExtractionResult::new("c0");
    r0.entities
        .push(entity("Topic", "fact 0 about topic alpha", "c0"));
    merger.merge(vec![r0]).await.unwrap();

    for i in 1..7 {
        let mut r = ExtractionResult::new(format!("c{i}"));
        r.entities.push(entity(
            "Topic",
            &format!("fact {i} about topic alpha"),
            &format!("c{i}"),
        ));
        merger.merge(vec![r]).await.unwrap();
    }

    assert_eq!(backend.entity_calls.load(Ordering::SeqCst), 0);

    let nodes = graph.get_nodes_batch(&["TOPIC".into()]).await.unwrap();
    let node = nodes.get("TOPIC").expect("TOPIC node");
    let desc = node
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        desc.contains(GRAPH_FIELD_SEP),
        "expected SEP-joined description, got: {desc}"
    );
    assert!(
        !desc.starts_with("LLM_ENTITY"),
        "LLM must not run below threshold: {desc}"
    );
    let n_frags = desc
        .split(GRAPH_FIELD_SEP)
        .filter(|s| !s.trim().is_empty())
        .count();
    assert_eq!(n_frags, 7);
}

#[tokio::test]
async fn e2e_entity_merge_calls_llm_at_eight_fragments() {
    let backend = Arc::new(CountingMergeBackend::new());
    let (merger, graph) = setup_merger(8, backend.clone()).await;

    let mut r0 = ExtractionResult::new("c0");
    r0.entities
        .push(entity("Widget", "fact 0 about widget beta", "c0"));
    merger.merge(vec![r0]).await.unwrap();

    for i in 1..7 {
        let mut r = ExtractionResult::new(format!("c{i}"));
        r.entities.push(entity(
            "Widget",
            &format!("fact {i} about widget beta"),
            &format!("c{i}"),
        ));
        merger.merge(vec![r]).await.unwrap();
    }
    assert_eq!(backend.entity_calls.load(Ordering::SeqCst), 0);

    let mut r7 = ExtractionResult::new("c7");
    r7.entities
        .push(entity("Widget", "fact 7 about widget beta", "c7"));
    merger.merge(vec![r7]).await.unwrap();

    assert_eq!(backend.entity_calls.load(Ordering::SeqCst), 1);

    let nodes = graph.get_nodes_batch(&["WIDGET".into()]).await.unwrap();
    let desc = nodes
        .get("WIDGET")
        .expect("WIDGET node")
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(desc, "LLM_ENTITY(8)");
}

#[tokio::test]
async fn e2e_relationship_merge_joins_below_eight_without_llm() {
    let backend = Arc::new(CountingMergeBackend::new());
    let (merger, graph) = setup_merger(8, backend.clone()).await;

    let mut r0 = ExtractionResult::new("c0");
    r0.entities.push(entity("Src", "source node", "c0"));
    r0.entities.push(entity("Tgt", "target node", "c0"));
    r0.relationships
        .push(rel("Src", "Tgt", "rel fact 0 between ends", "c0"));
    merger.merge(vec![r0]).await.unwrap();

    for i in 1..5 {
        let mut r = ExtractionResult::new(format!("c{i}"));
        r.entities
            .push(entity("Src", "source node", &format!("c{i}")));
        r.entities
            .push(entity("Tgt", "target node", &format!("c{i}")));
        r.relationships.push(rel(
            "Src",
            "Tgt",
            &format!("rel fact {i} between ends"),
            &format!("c{i}"),
        ));
        merger.merge(vec![r]).await.unwrap();
    }

    assert_eq!(backend.rel_calls.load(Ordering::SeqCst), 0);

    let edges = graph
        .get_edges_for_nodes_batch(&["SRC".into(), "TGT".into()])
        .await
        .unwrap();
    let edge = edges
        .iter()
        .find(|e| {
            (e.source.contains("SRC") && e.target.contains("TGT"))
                || (e.source.contains("TGT") && e.target.contains("SRC"))
        })
        .expect("SRC-TGT edge");
    let desc = edge
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        desc.contains(GRAPH_FIELD_SEP) || desc.contains("rel fact"),
        "expected joined rel description, got: {desc}"
    );
    assert!(!desc.starts_with("LLM_REL"));
}
