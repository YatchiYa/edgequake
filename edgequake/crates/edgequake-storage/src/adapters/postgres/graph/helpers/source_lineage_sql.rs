//! Document-scoped source lineage SQL predicates (SPEC-021 P-A3 / SPEC-045).
//!
//! SSOT for matching AGE node/edge properties against a document chunk prefix.
//! Two-path design (issue #305/#309):
//! - **Modern**: GIN-friendly `source_ids @>` exact candidates (indexed).
//! - **Legacy**: bounded LIKE/unnest only when modern arrays are absent.

/// Max chunk indices probed for GIN `@>` entity-count reconcile (list hot path).
///
/// WHY: `LIKE '%…%'` / `jsonb_array_elements_text` forces a Seq Scan over
/// `_ag_label_vertex` (~140k+ nodes → multi-second Documents list). Exact
/// chunk-id containment uses `idx_*_source_ids_gin` (~1 ms). Documents with
/// more than this many chunks still get a correct lower bound; stats write
/// path (P-A1) remains the primary count source.
pub(in crate::adapters::postgres::graph) const SOURCE_CHUNK_PROBE_LIMIT: usize = 256;

use super::escape::escape_sql_literal;

/// Normalize a document / chunk prefix to the `{doc_id}-chunk-` form.
///
/// Accepts either a bare document id or an already-suffixed
/// [`crate::kv_keys::doc_chunk_prefix`] value.
pub(in crate::adapters::postgres::graph) fn normalize_doc_chunk_prefix(prefix: &str) -> String {
    if prefix.ends_with("-chunk-") {
        prefix.to_string()
    } else {
        format!("{prefix}-chunk-")
    }
}

/// Build concrete chunk-id candidates for GIN `@>` probes (`{prefix}0`..`N-1`).
///
/// Batched list counts use SQL `generate_series` instead; this helper remains
/// for single-prefix probes, scan predicates, and tests.
#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::adapters::postgres::graph) fn source_chunk_id_candidates(
    prefix: &str,
    limit: usize,
) -> Vec<String> {
    let chunk_prefix = normalize_doc_chunk_prefix(prefix);
    let n = limit.clamp(1, SOURCE_CHUNK_PROBE_LIMIT);
    (0..n).map(|i| format!("{chunk_prefix}{i}")).collect()
}

/// Modern indexed path: `source_ids` / `source_chunk_ids` containment via `@>`.
///
/// Does **not** include LIKE — planners can use GIN without SeqScan fallback.
pub(in crate::adapters::postgres::graph) fn jsonb_matches_doc_source_prefix_modern(
    props: &str,
    doc_prefix: &str,
) -> String {
    let esc = escape_sql_literal(doc_prefix);
    let chunk = escape_sql_literal(&normalize_doc_chunk_prefix(doc_prefix));
    let mut parts = vec![format!(
        "({props}->'source_ids') @> to_jsonb('{esc}'::text)"
    )];
    // Probe first N chunk ids for GIN hits (delete cascade documents rarely exceed this).
    for i in 0..16 {
        parts.push(format!(
            "({props}->'source_ids') @> to_jsonb(('{chunk}' || '{i}')::text)"
        ));
        parts.push(format!(
            "({props}->'source_chunk_ids') @> to_jsonb(('{chunk}' || '{i}')::text)"
        ));
    }
    parts.push(format!(
        "({props}->'source_chunk_ids') @> to_jsonb('{esc}'::text)"
    ));
    format!("({})", parts.join(" OR "))
}

/// Legacy-only path: pipe `source_id` / LIKE / unnest when modern arrays are absent.
///
/// Bounded to rows without usable `source_ids` arrays so wipe-all never needs this
/// and cascade discovery does not SeqScan the whole modern graph.
pub(in crate::adapters::postgres::graph) fn jsonb_matches_doc_source_prefix_legacy(
    props: &str,
    doc_prefix: &str,
) -> String {
    let esc = escape_sql_literal(doc_prefix);
    let chunk = escape_sql_literal(&normalize_doc_chunk_prefix(doc_prefix));
    format!(
        "((jsonb_typeof({props}->'source_ids') IS DISTINCT FROM 'array' \
          OR jsonb_array_length(COALESCE({props}->'source_ids', '[]'::jsonb)) = 0) \
         AND ( \
           {props}->>'source_id' = '{esc}' \
           OR {props}->>'source_id' LIKE '{esc}%' \
           OR {props}->>'source_id' LIKE '%|{esc}%' \
           OR {props}->>'source_id' LIKE '%|{chunk}%' \
           OR {props}->>'source_id' LIKE '{chunk}%' \
           OR EXISTS ( \
               SELECT 1 FROM jsonb_array_elements_text( \
                   CASE \
                       WHEN jsonb_typeof({props}->'source_chunk_ids') = 'array' \
                       THEN {props}->'source_chunk_ids' \
                       ELSE '[]'::jsonb \
                   END \
               ) src \
               WHERE src LIKE '{esc}%' OR src LIKE '{chunk}%' OR src = '{esc}' \
           ) \
         ))",
        props = props,
        esc = esc,
        chunk = chunk,
    )
}

/// Combined predicate (compat / unit tests). Prefer two-path queries in `scan_ops`.
#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::adapters::postgres::graph) fn jsonb_matches_doc_source_prefix(
    props: &str,
    doc_prefix: &str,
) -> String {
    format!(
        "({} OR {})",
        jsonb_matches_doc_source_prefix_modern(props, doc_prefix),
        jsonb_matches_doc_source_prefix_legacy(props, doc_prefix),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_path_is_gin_only() {
        let sql = jsonb_matches_doc_source_prefix_modern("props", "doc-abc");
        assert!(sql.contains("@>"));
        assert!(!sql.contains("LIKE"));
        assert!(sql.contains("doc-abc-chunk-"));
    }

    #[test]
    fn legacy_path_requires_empty_source_ids() {
        let sql = jsonb_matches_doc_source_prefix_legacy("props", "doc-abc");
        assert!(sql.contains("LIKE"));
        assert!(sql.contains("jsonb_typeof"));
        assert!(sql.contains("source_chunk_ids"));
    }

    #[test]
    fn includes_source_chunk_ids_array_path() {
        let sql = jsonb_matches_doc_source_prefix("props", "doc-abc");
        assert!(sql.contains("source_chunk_ids"));
        assert!(sql.contains("source_ids"));
        assert!(sql.contains("doc-abc-chunk-"));
    }

    #[test]
    fn normalize_accepts_bare_doc_id_and_chunk_prefix() {
        assert_eq!(normalize_doc_chunk_prefix("doc-abc"), "doc-abc-chunk-");
        assert_eq!(
            normalize_doc_chunk_prefix("doc-abc-chunk-"),
            "doc-abc-chunk-"
        );
    }

    #[test]
    fn chunk_candidates_are_gin_friendly_exact_ids() {
        let ids = source_chunk_id_candidates("doc-abc-chunk-", 3);
        assert_eq!(
            ids,
            vec![
                "doc-abc-chunk-0".to_string(),
                "doc-abc-chunk-1".to_string(),
                "doc-abc-chunk-2".to_string(),
            ]
        );
    }
}
