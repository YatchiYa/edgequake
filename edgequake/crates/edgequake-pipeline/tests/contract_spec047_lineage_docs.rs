//! SPEC-047 / 021 L-A1 — ingest stamps/merges `source_document_ids[]`.

use std::collections::HashMap;

use edgequake_pipeline::merger::lineage::{
    merge_and_insert_document_lineage, resolve_incoming_document_ids,
    source_document_ids_from_properties,
};
use serde_json::Value;

#[test]
fn contract_cross_doc_entity_stores_document_union() {
    let mut props: HashMap<String, Value> = HashMap::new();

    // First document merge
    merge_and_insert_document_lineage(&mut props, Some("doc-a"), &["doc-a-chunk-0".into()]);
    let after_a = source_document_ids_from_properties(&props);
    assert_eq!(after_a, vec!["doc-a".to_string()]);
    assert_eq!(
        props.get("source_document_id").and_then(|v| v.as_str()),
        Some("doc-a")
    );

    // Second document merge (same entity, different PDF)
    merge_and_insert_document_lineage(&mut props, Some("doc-b"), &["doc-b-chunk-2".into()]);
    let after_b = source_document_ids_from_properties(&props);
    assert_eq!(after_b.len(), 2, "cross-doc entity must keep union");
    assert!(after_b.contains(&"doc-a".to_string()));
    assert!(after_b.contains(&"doc-b".to_string()));

    // Plural array written on node
    let plural = props
        .get("source_document_ids")
        .and_then(|v| v.as_array())
        .expect("source_document_ids must be written");
    assert_eq!(plural.len(), 2);
}

#[test]
fn contract_derive_docs_from_chunk_ids_when_singular_unset() {
    let incoming = resolve_incoming_document_ids(
        None,
        &["uuid-aaa-chunk-0".into(), "uuid-bbb-chunk-1".into()],
    );
    assert_eq!(
        incoming,
        vec!["uuid-aaa".to_string(), "uuid-bbb".to_string()]
    );
}

#[test]
fn contract_chunk_prefix_alone_seeds_plural_on_create() {
    let mut props = HashMap::new();
    merge_and_insert_document_lineage(&mut props, None, &["doc-x-chunk-7".into()]);
    assert_eq!(
        source_document_ids_from_properties(&props),
        vec!["doc-x".to_string()]
    );
}
