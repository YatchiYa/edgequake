//! SPEC-047 P6: unique-before-embed + parallel sub-batches (LightRAG alignment).
//!
//! Contract: entity/relationship embed cost is O(unique names), not O(mentions).

use edgequake_pipeline::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};
use edgequake_pipeline::pipeline::unique_embed::{
    entity_embed_text, unique_entities_for_embed, unique_relationships_for_embed,
};

fn mention(name: &str, desc: &str) -> ExtractedEntity {
    ExtractedEntity {
        name: name.into(),
        entity_type: "CONCEPT".into(),
        description: desc.into(),
        importance: 0.5,
        source_spans: vec![],
        embedding: None,
        source_chunk_ids: vec![],
        source_document_id: None,
        source_file_path: None,
    }
}

#[test]
fn contract_unique_before_embed_collapses_mentions() {
    let mut chunks = Vec::new();
    for i in 0..50 {
        let mut r = ExtractionResult::new(format!("c{i}"));
        r.entities = vec![
            mention("Shared Entity", &format!("desc from chunk {i}")),
            mention(&format!("Unique {i}"), "once"),
        ];
        chunks.push(r);
    }
    let unique = unique_entities_for_embed(&chunks);
    // 1 shared + 50 unique = 51, not 100 mentions
    assert_eq!(unique.len(), 51);
    let shared = unique.iter().find(|u| u.key == "SHARED_ENTITY").unwrap();
    assert_eq!(shared.mentions.len(), 50);
    assert!(shared.text.contains("desc from chunk"));
}

#[test]
fn contract_entity_embed_text_matches_lightrag_newline_format() {
    let text = entity_embed_text("FOO", "bar description");
    assert_eq!(text, "FOO\nbar description");
    assert!(!text.contains(": "));
}

#[test]
fn contract_unique_relationships_collapse_cross_chunk() {
    let mut chunks = Vec::new();
    for i in 0..10 {
        let mut r = ExtractionResult::new(format!("c{i}"));
        r.relationships = vec![ExtractedRelationship {
            source: "A".into(),
            target: "B".into(),
            relation_type: "RELATED".into(),
            description: format!("rel {i}"),
            keywords: vec!["k".into()],
            weight: 1.0,
            embedding: None,
            source_chunk_id: None,
            source_document_id: None,
            source_file_path: None,
        }];
        chunks.push(r);
    }
    let unique = unique_relationships_for_embed(&chunks);
    assert_eq!(unique.len(), 1);
    assert_eq!(unique[0].mentions.len(), 10);
}
