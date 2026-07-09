//! Graph node source lineage properties (SPEC-045 / BR0007).
//!
//! SSOT: pipeline writes both `source_chunk_ids` (citation tracking) and
//! `source_ids` (analytics / reconcile / cascade-delete predicates).

use std::collections::HashMap;

use serde_json::Value;

/// Mirror chunk lineage into graph node properties for read-path compatibility.
pub fn insert_chunk_lineage_properties(
    properties: &mut HashMap<String, Value>,
    chunk_ids: &[String],
) {
    let json = Value::Array(chunk_ids.iter().cloned().map(Value::String).collect());
    properties.insert("source_chunk_ids".to_string(), json.clone());
    properties.insert("source_ids".to_string(), json);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_chunk_ids_to_source_ids() {
        let mut props = HashMap::new();
        insert_chunk_lineage_properties(&mut props, &["doc-a-chunk-0".into()]);
        assert_eq!(
            props.get("source_chunk_ids"),
            props.get("source_ids"),
            "source_ids must mirror source_chunk_ids"
        );
    }
}
