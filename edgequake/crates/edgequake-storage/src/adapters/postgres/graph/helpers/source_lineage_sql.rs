//! Document-scoped source lineage SQL predicates (SPEC-021 P-A3 / SPEC-045).
//!
//! SSOT for matching AGE node/edge properties against a document chunk prefix.
//! Covers legacy `source_id`, modern `source_ids`, and pipeline `source_chunk_ids`.

/// Escape a string for safe inclusion in SQL single-quoted literals.
pub(in crate::adapters::postgres::graph) fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Build a SQL predicate matching jsonb `properties` against one document prefix.
///
/// `props` must already be a jsonb expression (e.g. `(agtype_to_json(v.properties))::jsonb`).
/// `doc_prefix` is the document id (used to derive `{doc_id}-chunk-` patterns).
pub(in crate::adapters::postgres::graph) fn jsonb_matches_doc_source_prefix(
    props: &str,
    doc_prefix: &str,
) -> String {
    let esc = escape_sql_literal(doc_prefix);
    let chunk = escape_sql_literal(&format!("{doc_prefix}-chunk-"));

    format!(
        "({props}->>'source_id' LIKE '{esc}%' \
         OR {props}->>'source_id' LIKE '%|{esc}%' \
         OR {props}->>'source_id' LIKE '%|{chunk}%' \
         OR {props}->>'source_id' LIKE '{chunk}%' \
         OR EXISTS ( \
             SELECT 1 FROM jsonb_array_elements_text( \
                 CASE \
                     WHEN jsonb_typeof({props}->'source_ids') = 'array' \
                     THEN {props}->'source_ids' \
                     ELSE '[]'::jsonb \
                 END \
             ) src \
             WHERE src LIKE '{esc}%' OR src LIKE '{chunk}%' OR src = '{esc}' \
         ) \
         OR EXISTS ( \
             SELECT 1 FROM jsonb_array_elements_text( \
                 CASE \
                     WHEN jsonb_typeof({props}->'source_chunk_ids') = 'array' \
                     THEN {props}->'source_chunk_ids' \
                     ELSE '[]'::jsonb \
                 END \
             ) src \
             WHERE src LIKE '{esc}%' OR src LIKE '{chunk}%' OR src = '{esc}' \
         ))",
        props = props,
        esc = esc,
        chunk = chunk,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_source_chunk_ids_array_path() {
        let sql = jsonb_matches_doc_source_prefix("props", "doc-abc");
        assert!(sql.contains("source_chunk_ids"));
        assert!(sql.contains("source_ids"));
        assert!(sql.contains("doc-abc-chunk-"));
    }
}
