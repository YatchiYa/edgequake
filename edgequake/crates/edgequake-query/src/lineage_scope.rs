//! SPEC-047 / 021 — Document-scope lineage helpers (DRY).
//!
//! Law (L1–L4):
//! - Page lives on chunks; entities resolve page via chunk ids.
//! - Multi-doc entities keep a **union** of document parents.
//! - Under active scope: keep iff lineage intersects allowed docs.
//! - Derive docs from `*-chunk-N` ids when plural/singular unset.
//! - Truly unknown provenance under scope → **drop** (fail-closed).

use std::collections::HashSet;

use crate::helpers::extract_document_id;

/// Unique document IDs implied by EdgeQuake chunk id convention (`{doc}-chunk-N`).
pub fn document_ids_from_chunk_ids(chunk_ids: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for cid in chunk_ids {
        if let Some(doc) = extract_document_id(cid) {
            if seen.insert(doc.clone()) {
                out.push(doc);
            }
        }
    }
    out
}

/// Resolve effective document parents for an entity/rel (021 L2).
///
/// Priority: explicit plural → singular → derived from chunk ids.
pub fn resolve_lineage_document_ids(
    source_document_ids: &[String],
    source_document_id: Option<&str>,
    source_chunk_ids: &[String],
) -> Vec<String> {
    if !source_document_ids.is_empty() {
        return source_document_ids.to_vec();
    }
    if let Some(id) = source_document_id {
        if !id.is_empty() {
            return vec![id.to_string()];
        }
    }
    document_ids_from_chunk_ids(source_chunk_ids)
}

/// Whether lineage intersects the allowed document set (021 L3).
///
/// - Empty `lineage_docs` under active scope → **false** (L4 fail-closed).
/// - Non-empty → keep if ANY id ∈ allowed.
pub fn lineage_intersects_allowed(lineage_docs: &[String], allowed: &HashSet<&str>) -> bool {
    if lineage_docs.is_empty() {
        return false;
    }
    lineage_docs.iter().any(|id| allowed.contains(id.as_str()))
}

/// Filter chunk ids to those whose derived document id is in `allowed`.
///
/// When `allowed` is `None` or empty-as-no-scope is not used — caller passes
/// `Some` only when document scope is active. Empty allowed set → all dropped.
pub fn filter_chunk_ids_by_allowed_docs(
    chunk_ids: &[String],
    allowed_document_ids: Option<&[String]>,
) -> Vec<String> {
    let Some(allowed) = allowed_document_ids else {
        return chunk_ids.to_vec();
    };
    let id_set: HashSet<&str> = allowed.iter().map(|s| s.as_str()).collect();
    chunk_ids
        .iter()
        .filter(|cid| {
            extract_document_id(cid)
                .map(|d| id_set.contains(d.as_str()))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_ids_from_chunk_ids_dedupes() {
        let ids = vec![
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee-chunk-0".into(),
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee-chunk-1".into(),
            "ffffffff-bbbb-cccc-dddd-eeeeeeeeeeee-chunk-0".into(),
        ];
        let docs = document_ids_from_chunk_ids(&ids);
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0], "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
    }

    #[test]
    fn resolve_prefers_plural_then_singular_then_chunks() {
        let plural = resolve_lineage_document_ids(
            &["doc-a".into(), "doc-b".into()],
            Some("doc-x"),
            &["doc-z-chunk-0".into()],
        );
        assert_eq!(plural, vec!["doc-a", "doc-b"]);

        let singular = resolve_lineage_document_ids(&[], Some("doc-a"), &[]);
        assert_eq!(singular, vec!["doc-a"]);

        let derived = resolve_lineage_document_ids(&[], None, &["doc-a-chunk-3".into()]);
        assert_eq!(derived, vec!["doc-a"]);
    }

    #[test]
    fn fail_closed_when_no_lineage() {
        let allowed: HashSet<&str> = ["doc-a"].into_iter().collect();
        assert!(!lineage_intersects_allowed(&[], &allowed));
        assert!(lineage_intersects_allowed(&["doc-a".into()], &allowed));
        assert!(!lineage_intersects_allowed(&["doc-z".into()], &allowed));
    }

    #[test]
    fn filter_chunk_ids_respects_scope() {
        let ids = vec![
            "doc-a-chunk-0".into(),
            "doc-b-chunk-1".into(),
            "orphan".into(),
        ];
        let allowed = vec!["doc-a".to_string()];
        let filtered = filter_chunk_ids_by_allowed_docs(&ids, Some(&allowed));
        assert_eq!(filtered, vec!["doc-a-chunk-0".to_string()]);
        assert_eq!(filter_chunk_ids_by_allowed_docs(&ids, None), ids);
    }
}
