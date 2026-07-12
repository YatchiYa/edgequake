//! SPEC-047 / 021 — Lineage scope (Entity → Chunk → Document) e2e.
//!
//! Laws under test:
//! - L2: multi-doc entity kept when ANY parent intersects allowed
//! - L3: foreign-only lineage dropped
//! - L4: unknown provenance under scope → drop
//! - L-A2: derive docs from `{doc}-chunk-N` when plural/singular unset
//! - L-A3: kg_chunk_pick never returns out-of-scope chunk ids when scoped

use edgequake_query::context::{QueryContext, RetrievedEntity, RetrievedRelationship};
use edgequake_query::context_filter::filter_context_by_document_ids;
use edgequake_query::kg_chunk_pick::{collect_kg_chunk_ids, collect_kg_chunk_ids_scoped};
use edgequake_query::lineage_scope::{
    document_ids_from_chunk_ids, filter_chunk_ids_by_allowed_docs, resolve_lineage_document_ids,
};

#[test]
fn e2e_l2_multi_doc_entity_kept_when_any_allowed() {
    let mut ctx = QueryContext::new();
    ctx.add_entity(
        RetrievedEntity::new("Shared", "ORG", "appears in two PDFs")
            .with_source_document_ids(vec!["doc-a".into(), "doc-b".into()])
            .with_source_chunk_ids(vec!["doc-a-chunk-0".into(), "doc-b-chunk-1".into()]),
    );
    ctx.add_entity(
        RetrievedEntity::new("Foreign", "ORG", "other corpus")
            .with_source_document_ids(vec!["doc-z".into()]),
    );

    filter_context_by_document_ids(&mut ctx, Some(&["doc-a".to_string()]));
    assert_eq!(ctx.entities.len(), 1);
    assert_eq!(ctx.entities[0].name, "Shared");
}

#[test]
fn e2e_l4_unknown_provenance_dropped_under_scope() {
    let mut ctx = QueryContext::new();
    ctx.add_entity(RetrievedEntity::new("Orphan", "CONCEPT", "no lineage"));
    ctx.add_relationship(RetrievedRelationship::new("X", "Y", "REL"));

    filter_context_by_document_ids(&mut ctx, Some(&["doc-a".to_string()]));
    assert!(ctx.entities.is_empty());
    assert!(ctx.relationships.is_empty());
}

#[test]
fn e2e_la2_derive_document_from_chunk_ids() {
    let mut ctx = QueryContext::new();
    // No plural/singular — only chunk ids (legacy AGE nodes)
    ctx.add_entity(
        RetrievedEntity::new("Legacy", "PERSON", "d")
            .with_source_chunk_ids(vec!["doc-a-chunk-3".into(), "doc-z-chunk-0".into()]),
    );
    ctx.add_relationship(
        RetrievedRelationship::new("Legacy", "Other", "WORKS_WITH")
            .with_source_chunk_id("doc-a-chunk-3"),
    );

    filter_context_by_document_ids(&mut ctx, Some(&["doc-a".to_string()]));
    assert_eq!(ctx.entities.len(), 1);
    assert_eq!(ctx.relationships.len(), 1);

    let docs =
        resolve_lineage_document_ids(&[], None, &["doc-a-chunk-3".into(), "doc-z-chunk-0".into()]);
    assert!(docs.contains(&"doc-a".to_string()));
    assert!(docs.contains(&"doc-z".to_string()));
}

#[test]
fn e2e_la2_foreign_chunk_lineage_excluded() {
    let mut ctx = QueryContext::new();
    ctx.add_entity(
        RetrievedEntity::new("Leak", "ORG", "d")
            .with_source_chunk_ids(vec!["doc-z-chunk-0".into()]),
    );
    filter_context_by_document_ids(&mut ctx, Some(&["doc-a".to_string()]));
    assert!(
        ctx.entities.is_empty(),
        "must not leak foreign chunk lineage"
    );
}

#[test]
fn e2e_la3_kg_chunk_pick_intersects_allowed_docs() {
    let mut ctx = QueryContext::new();
    let mut e = RetrievedEntity::new("E", "ORG", "d");
    e.source_chunk_ids = vec![
        "doc-a-chunk-0".into(),
        "doc-a-chunk-1".into(),
        "doc-z-chunk-99".into(),
    ];
    ctx.add_entity(e);
    ctx.add_relationship(
        RetrievedRelationship::new("E", "F", "REL").with_source_chunk_id("doc-z-chunk-1"),
    );

    let unscoped = collect_kg_chunk_ids(&ctx, 0);
    assert!(unscoped.iter().any(|id| id.contains("doc-z")));

    let allowed = vec!["doc-a".to_string()];
    let scoped = collect_kg_chunk_ids_scoped(&ctx, 0, Some(&allowed));
    assert!(!scoped.is_empty());
    assert!(
        scoped.iter().all(|id| id.starts_with("doc-a-")),
        "scoped pick must not fetch foreign chunks: {scoped:?}"
    );
    assert!(!scoped.iter().any(|id| id.contains("doc-z")));
}

#[test]
fn e2e_la3_weight_candidates_filter_helper() {
    let ids = vec![
        "doc-a-chunk-0".into(),
        "doc-b-chunk-1".into(),
        "orphan".into(),
    ];
    let filtered = filter_chunk_ids_by_allowed_docs(&ids, Some(&["doc-b".to_string()]));
    assert_eq!(filtered, vec!["doc-b-chunk-1".to_string()]);
    assert_eq!(document_ids_from_chunk_ids(&ids).len(), 2);
}

#[test]
fn e2e_none_scope_is_noop_for_filter() {
    let mut ctx = QueryContext::new();
    ctx.add_entity(RetrievedEntity::new("Keep", "T", "d"));
    filter_context_by_document_ids(&mut ctx, None);
    assert_eq!(ctx.entities.len(), 1);
}
