//! SPEC-047 / 021 L-A4 — Doc-diverse KEEP does not wipe minority documents.
//!
//! Law L6: when SOURCE_IDS cap saturates, retain ≥1 chunk per contributing doc
//! when possible (round-robin, oldest-first within each doc).

use std::collections::HashSet;

use edgequake_pipeline::{
    apply_source_ids_limit, document_id_from_chunk_id, document_ids_from_chunk_ids,
    merge_and_insert_document_lineage, merge_source_ids, source_document_ids_from_properties,
    truncate_keep_doc_diverse, SourceIdsLimitMethod,
};
use std::collections::HashMap;

#[test]
fn e2e_la4_naive_keep_would_wipe_minority_but_diverse_keeps_it() {
    let mut ids = Vec::new();
    for i in 0..10 {
        ids.push(format!("doc-majority-chunk-{i}"));
    }
    ids.push("doc-minority-chunk-0".into());
    ids.push("doc-minority-chunk-1".into());

    // Classic head KEEP (what we used to do) — wipe minority.
    let naive_head: Vec<_> = ids.iter().take(5).cloned().collect();
    let naive_docs: HashSet<_> = naive_head
        .iter()
        .filter_map(|id| document_id_from_chunk_id(id))
        .collect();
    assert!(
        !naive_docs.contains("doc-minority"),
        "precondition: naive head wipes minority"
    );

    let diverse = truncate_keep_doc_diverse(&ids, 5);
    assert_eq!(diverse.len(), 5);
    let docs: HashSet<_> = diverse
        .iter()
        .filter_map(|id| document_id_from_chunk_id(id))
        .collect();
    assert!(docs.contains("doc-majority"));
    assert!(
        docs.contains("doc-minority"),
        "L-A4: minority must survive; got {diverse:?}"
    );
}

#[test]
fn e2e_la4_apply_keep_path_uses_diversity() {
    let mut ids = Vec::new();
    for i in 0..8 {
        ids.push(format!("doc-a-chunk-{i}"));
    }
    for i in 0..3 {
        ids.push(format!("doc-b-chunk-{i}"));
    }
    ids.push("doc-c-chunk-0".into());

    let capped = apply_source_ids_limit(&ids, 6, SourceIdsLimitMethod::Keep);
    assert_eq!(capped.len(), 6);
    let docs = document_ids_from_chunk_ids(&capped);
    assert!(docs.contains(&"doc-a".to_string()));
    assert!(docs.contains(&"doc-b".to_string()));
    assert!(
        docs.contains(&"doc-c".to_string()),
        "third parent must get a slot under round-robin; docs={docs:?} capped={capped:?}"
    );
}

#[test]
fn e2e_la4_merge_then_cap_preserves_cross_doc_lineage() {
    // Simulate entity seen in doc-a (many chunks) then doc-b (few).
    let existing: Vec<_> = (0..50).map(|i| format!("doc-a-chunk-{i}")).collect();
    let incoming = vec!["doc-b-chunk-0".into(), "doc-b-chunk-1".into()];
    let merged = merge_source_ids(&existing, &incoming);
    let capped = apply_source_ids_limit(&merged, 10, SourceIdsLimitMethod::Keep);

    let mut props = HashMap::new();
    merge_and_insert_document_lineage(&mut props, None, &capped);
    let docs = source_document_ids_from_properties(&props);
    assert!(docs.contains(&"doc-a".to_string()));
    assert!(
        docs.contains(&"doc-b".to_string()),
        "capped chunks must still seed both document parents; docs={docs:?}"
    );
}

#[test]
fn e2e_la4_fifo_still_takes_newest_tail() {
    let ids: Vec<_> = (0..5)
        .map(|i| format!("doc-x-chunk-{i}"))
        .chain((0..2).map(|i| format!("doc-y-chunk-{i}")))
        .collect();
    let capped = apply_source_ids_limit(&ids, 3, SourceIdsLimitMethod::Fifo);
    // Tail of 7-length list: indices 4,5,6 → doc-x-chunk-4, doc-y-chunk-0, doc-y-chunk-1
    assert_eq!(
        capped,
        vec![
            "doc-x-chunk-4".to_string(),
            "doc-y-chunk-0".to_string(),
            "doc-y-chunk-1".to_string()
        ]
    );
}
