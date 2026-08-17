//! SPEC-047 P7b/c/d e2e: parallel merge + KEEP skip + soft-resume no-LLM.
//!
//! Uses in-memory graph + counting [`DescriptionMergeBackend`] (SOLID DI).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use edgequake_pipeline::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};
use edgequake_pipeline::{
    DescriptionMergeBackend, KnowledgeGraphMerger, MergerConfig, Result as PipelineResult,
    SourceIdsLimitMethod, GRAPH_FIELD_SEP,
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
    config: MergerConfig,
    backend: Arc<CountingMergeBackend>,
) -> (
    KnowledgeGraphMerger<MemoryGraphStorage, MemoryVectorStorage>,
    Arc<MemoryGraphStorage>,
) {
    let graph = Arc::new(MemoryGraphStorage::new("p7bcd-e2e"));
    let vector = Arc::new(MemoryVectorStorage::new("p7bcd-e2e", EMBED_DIM));
    graph.initialize().await.unwrap();
    vector.initialize().await.unwrap();
    let merger =
        KnowledgeGraphMerger::new(config, graph.clone(), vector).with_merge_backend(backend);
    (merger, graph)
}

fn base_config() -> MergerConfig {
    MergerConfig {
        use_llm_summarization: true,
        force_llm_summary_on_merge: 8,
        summary_max_tokens: 12_000,
        description_similarity_threshold: 1.01,
        merge_max_async: 4,
        source_ids_limit_method: SourceIdsLimitMethod::Keep,
        max_source_ids_per_entity: 200,
        max_source_ids_per_relation: 200,
        ..MergerConfig::default()
    }
}

/// P7c: soft-resume with existing edge + <8 fragments → 0 LLM calls.
#[tokio::test]
async fn e2e_p7c_soft_resume_existing_edge_below_force_no_llm() {
    let backend = Arc::new(CountingMergeBackend::new());
    let (merger, _graph) = setup_merger(base_config(), backend.clone()).await;

    let mut r0 = ExtractionResult::new("c0");
    r0.entities
        .push(entity("Alpha", "alpha seed fact zero", "c0"));
    r0.entities
        .push(entity("Beta", "beta seed fact zero", "c0"));
    r0.relationships
        .push(rel("Alpha", "Beta", "alpha relates to beta zero", "c0"));
    merger.merge(vec![r0]).await.unwrap();

    // Soft-resume: same endpoints, distinct short facts (< force=8).
    for i in 1..5 {
        let mut r = ExtractionResult::new(format!("c{i}"));
        r.entities.push(entity(
            "Alpha",
            &format!("alpha soft resume fact {i}"),
            &format!("c{i}"),
        ));
        r.entities.push(entity(
            "Beta",
            &format!("beta soft resume fact {i}"),
            &format!("c{i}"),
        ));
        r.relationships.push(rel(
            "Alpha",
            "Beta",
            &format!("alpha relates soft {i}"),
            &format!("c{i}"),
        ));
        merger.merge(vec![r]).await.unwrap();
    }

    assert_eq!(
        backend.entity_calls.load(Ordering::SeqCst),
        0,
        "P7c: entity soft-resume <8 frags must not LLM"
    );
    assert_eq!(
        backend.rel_calls.load(Ordering::SeqCst),
        0,
        "P7c: relationship soft-resume <8 frags must not LLM"
    );
}

/// P7d: KEEP saturated → skip description update (0 LLM, stats.skipped).
#[tokio::test]
async fn e2e_p7d_keep_skips_saturated_entity_update() {
    let backend = Arc::new(CountingMergeBackend::new());
    let mut config = base_config();
    config.max_source_ids_per_entity = 3;
    let (merger, graph) = setup_merger(config, backend.clone()).await;

    // Seed with 3 distinct chunks → saturated at max=3.
    for i in 0..3 {
        let mut r = ExtractionResult::new(format!("seed{i}"));
        r.entities.push(entity(
            "Saturated",
            &format!("seed description {i} unique"),
            &format!("seed{i}"),
        ));
        merger.merge(vec![r]).await.unwrap();
    }

    let before = graph
        .get_nodes_batch(&["SATURATED".into()])
        .await
        .unwrap()
        .get("SATURATED")
        .expect("node")
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let calls_before = backend.entity_calls.load(Ordering::SeqCst);

    // Brand-new chunk while saturated → KEEP skip.
    let mut r = ExtractionResult::new("brand-new");
    r.entities.push(entity(
        "Saturated",
        "should never appear in description",
        "brand-new",
    ));
    let stats = merger.merge(vec![r]).await.unwrap();
    assert!(
        stats.entities_skipped_saturated >= 1,
        "expected saturated skip, got {stats:?}"
    );
<<<<<<< HEAD
=======
    assert!(
        stats.entities_spine_ensured_saturated >= 1,
        "SPEC-098: saturated KEEP must ensure spine, got {stats:?}"
    );
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

    let after = graph
        .get_nodes_batch(&["SATURATED".into()])
        .await
        .unwrap()
        .get("SATURATED")
        .expect("node")
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    assert_eq!(before, after, "KEEP must not mutate description");
    assert!(
        !after.contains("should never appear"),
        "new chunk text leaked: {after}"
    );
    assert_eq!(
        backend.entity_calls.load(Ordering::SeqCst),
        calls_before,
        "KEEP skip must not call LLM"
    );
}

/// P7d: KEEP saturated relationship skip.
#[tokio::test]
async fn e2e_p7d_keep_skips_saturated_relationship_update() {
    let backend = Arc::new(CountingMergeBackend::new());
    let mut config = base_config();
    config.max_source_ids_per_relation = 2;
    let (merger, _graph) = setup_merger(config, backend.clone()).await;

    for i in 0..2 {
        let mut r = ExtractionResult::new(format!("rseed{i}"));
        r.entities
            .push(entity("Left", &format!("left {i}"), &format!("rseed{i}")));
        r.entities
            .push(entity("Right", &format!("right {i}"), &format!("rseed{i}")));
        r.relationships.push(rel(
            "Left",
            "Right",
            &format!("edge seed {i}"),
            &format!("rseed{i}"),
        ));
        merger.merge(vec![r]).await.unwrap();
    }

    let mut r = ExtractionResult::new("rnew");
    r.entities.push(entity("Left", "left new", "rnew"));
    r.entities.push(entity("Right", "right new", "rnew"));
    r.relationships
        .push(rel("Left", "Right", "edge must skip", "rnew"));
    let stats = merger.merge(vec![r]).await.unwrap();
    assert!(
        stats.relationships_skipped_saturated >= 1,
        "expected rel saturated skip, got {stats:?}"
    );
}

/// P7b: parallel merge of many unique entities preserves SEP join correctness.
#[tokio::test]
async fn e2e_p7b_parallel_unique_entity_merge_correct() {
    let backend = Arc::new(CountingMergeBackend::new());
    let mut config = base_config();
    config.merge_max_async = 8;
    let (merger, graph) = setup_merger(config, backend.clone()).await;

    // One batch with many unique entities (parallel create path).
    let mut r0 = ExtractionResult::new("batch0");
    for i in 0..16 {
        r0.entities.push(entity(
            &format!("Topic{i}"),
            &format!("topic {i} fact zero"),
            "batch0",
        ));
    }
    merger.merge(vec![r0]).await.unwrap();

    // Second pass: update all in parallel with distinct facts (still < force).
    let mut r1 = ExtractionResult::new("batch1");
    for i in 0..16 {
        r1.entities.push(entity(
            &format!("Topic{i}"),
            &format!("topic {i} fact one distinct"),
            "batch1",
        ));
    }
    let stats = merger.merge(vec![r1]).await.unwrap();
    assert_eq!(stats.entities_updated, 16);
    assert_eq!(backend.entity_calls.load(Ordering::SeqCst), 0);

    for i in 0..16 {
        let key = format!("TOPIC{i}");
        let nodes = graph
            .get_nodes_batch(std::slice::from_ref(&key))
            .await
            .unwrap();
        let desc = nodes
            .get(&key)
            .expect("node")
            .properties
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            desc.contains(GRAPH_FIELD_SEP),
            "expected SEP join for {key}: {desc}"
        );
        assert!(
            desc.contains("fact zero") && desc.contains("fact one"),
            "both fragments required for {key}: {desc}"
        );
    }
}
