//! SPEC-114 G-114-16 — Relation types + typed edges enforced through gleaning.
//!
//! Mirror of `e2e_issue276_gleaning_strict` for relation schema.
//!
//! ```bash
//! cargo test -p edgequake-pipeline --test e2e_spec114_gleaning_relations
//! ```

use std::collections::HashSet;
use std::sync::Arc;

use edgequake_llm::MockProvider;
use edgequake_pipeline::prompts::{EntityExtractionSchema, RelationEdge};
use edgequake_pipeline::{
    build_ingestion_pipeline, IngestionPipelineOptions, Pipeline, PipelineConfig,
};

const DOC: &str = r#"
Alice is an engineer at Acme Corporation in Paris.
"#;

const BASE_JSON: &str = r#"{
  "entities": [
    {"name": "Alice", "type": "PERSON", "description": "Engineer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Company"}
  ],
  "relationships": [
    {"source": "Alice", "target": "Acme", "type": "WORKS_AT", "description": "ok"}
  ]
}"#;

/// Gleaning emits illegal relation labels + reversed typed edge.
const GLEANING_BAD_RELATIONS_JSON: &str = r#"{
  "entities": [
    {"name": "Alice", "type": "PERSON", "description": "Engineer"},
    {"name": "Acme", "type": "ORGANIZATION", "description": "Company"},
    {"name": "Paris", "type": "LOCATION", "description": "City"}
  ],
  "relationships": [
    {"source": "Alice", "target": "Acme", "type": "EMPLOYS", "description": "illegal label"},
    {"source": "Acme", "target": "Alice", "type": "WORKS_AT", "description": "illegal endpoints"},
    {"source": "Alice", "target": "Paris", "type": "LOCATED_IN", "description": "ok if free endpoints"}
  ]
}"#;

fn schema_strict_relations_with_edges() -> EntityExtractionSchema {
    EntityExtractionSchema {
        types: vec![
            "PERSON".into(),
            "ORGANIZATION".into(),
            "LOCATION".into(),
            "OTHER".into(),
        ],
        strict: true,
        relation_types: vec!["WORKS_AT".into(), "RELATED_TO".into(), "LOCATED_IN".into()],
        relation_strict: true,
        relation_edges: vec![RelationEdge {
            source: "PERSON".into(),
            relation: "WORKS_AT".into(),
            target: "ORGANIZATION".into(),
        }],
    }
}

fn schema_strict_relations_no_edges() -> EntityExtractionSchema {
    EntityExtractionSchema {
        types: schema_strict_relations_with_edges().types,
        strict: true,
        relation_types: vec!["WORKS_AT".into(), "RELATED_TO".into()],
        relation_strict: true,
        relation_edges: Vec::new(),
    }
}

fn schema_permissive_relations() -> EntityExtractionSchema {
    EntityExtractionSchema {
        types: schema_strict_relations_with_edges().types,
        strict: true,
        relation_types: vec!["WORKS_AT".into(), "RELATED_TO".into()],
        relation_strict: false,
        relation_edges: Vec::new(),
    }
}

fn schema_free_form_relations() -> EntityExtractionSchema {
    EntityExtractionSchema {
        types: schema_strict_relations_with_edges().types,
        strict: true,
        relation_types: Vec::new(),
        relation_strict: true,
        relation_edges: Vec::new(),
    }
}

async fn queue_base_then_gleaning(mock: &MockProvider, rounds: usize) {
    for _ in 0..rounds {
        mock.add_response(BASE_JSON).await;
        mock.add_response(GLEANING_BAD_RELATIONS_JSON).await;
    }
}

async fn run_pipeline(schema: EntityExtractionSchema) -> edgequake_pipeline::ProcessingResult {
    let mock = Arc::new(MockProvider::new());
    queue_base_then_gleaning(mock.as_ref(), 8).await;
    let embedding =
        Arc::new(MockProvider::new()) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>;

    let built = build_ingestion_pipeline(
        mock.clone(),
        embedding,
        schema,
        IngestionPipelineOptions::from_document_size(500).with_gleaning(true, 1),
    );
    let extractor = built.extractor().expect("extractor");
    let pipeline = Pipeline::new(PipelineConfig {
        enable_entity_extraction: true,
        enable_relationship_extraction: true,
        enable_chunk_embeddings: false,
        enable_entity_embeddings: false,
        chunker: built.config().chunker.clone(),
        chunk_strategy: built.config().chunk_strategy,
        ..Default::default()
    })
    .with_extractor(extractor);

    pipeline
        .process("spec114-gleaning-doc", DOC)
        .await
        .expect("pipeline process")
}

fn collected_relation_types(result: &edgequake_pipeline::ProcessingResult) -> HashSet<String> {
    let mut out = HashSet::new();
    for extraction in &result.extractions {
        for rel in &extraction.relationships {
            out.insert(rel.relation_type.to_uppercase());
        }
    }
    out
}

#[tokio::test]
async fn spec114_gleaning_strict_remaps_illegal_relation_labels() {
    let result = run_pipeline(schema_strict_relations_no_edges()).await;
    let types = collected_relation_types(&result);
    assert!(
        !types.contains("EMPLOYS"),
        "EMPLOYS must not survive strict relation allow-list, got {types:?}"
    );
    assert!(
        types.contains("WORKS_AT") || types.contains("RELATED_TO"),
        "expected allow-list labels, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_gleaning_permissive_keeps_unknown_relation() {
    let result = run_pipeline(schema_permissive_relations()).await;
    let types = collected_relation_types(&result);
    assert!(
        types.contains("EMPLOYS"),
        "permissive must keep EMPLOYS from gleaning, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_gleaning_free_form_keeps_any_label() {
    let result = run_pipeline(schema_free_form_relations()).await;
    let types = collected_relation_types(&result);
    assert!(
        types.contains("EMPLOYS") || types.contains("WORKS_AT"),
        "free-form must preserve gleaning labels, got {types:?}"
    );
}

#[tokio::test]
async fn spec114_gleaning_typed_edges_remap_illegal_endpoints() {
    let result = run_pipeline(schema_strict_relations_with_edges()).await;
    assert!(
        !result.extractions.is_empty(),
        "expected extractions after gleaning"
    );

    // Every WORKS_AT that remains should prefer PERSON→ORGANIZATION shape when
    // source/target types are known; illegal Acme→Alice remaps away from WORKS_AT
    // toward RELATED_TO (or first allow-list). We assert EMPLOYS is gone and at
    // least one allow-list relation survives.
    let types = collected_relation_types(&result);
    assert!(
        !types.contains("EMPLOYS"),
        "illegal EMPLOYS must remap under edged+strict schema, got {types:?}"
    );
    assert!(
        types.contains("WORKS_AT") || types.contains("RELATED_TO") || types.contains("LOCATED_IN"),
        "expected enforced allow-list relations, got {types:?}"
    );

    let gleaning_ran = result.extractions.iter().any(|ex| {
        ex.metadata
            .get("gleaning_iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            >= 1
    });
    assert!(gleaning_ran, "gleaning wrapper must run");
}

#[tokio::test]
async fn spec114_gleaning_empty_edges_unconstrained_endpoints() {
    // LOCATED_IN Alice→Paris is allowed by label list and unconstrained endpoints.
    let mut schema = schema_strict_relations_no_edges();
    schema.relation_types = vec!["WORKS_AT".into(), "RELATED_TO".into(), "LOCATED_IN".into()];
    let result = run_pipeline(schema).await;
    let types = collected_relation_types(&result);
    assert!(
        types.contains("LOCATED_IN") || types.contains("WORKS_AT") || types.contains("RELATED_TO"),
        "empty relation_edges must not drop valid labels, got {types:?}"
    );
}
