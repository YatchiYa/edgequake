//! Rebuild entity/relationship descriptions from remaining chunk sources
//! (SPEC-046 EQ-046-12 / LightRAG `rebuild_knowledge_from_chunks` lite).
//!
//! First principle: after partial document delete, shared entities must keep
//! provenance **and** descriptions that no longer cite deleted chunks.
//!
//! This module is **LLM-free**: it filters pipe/newline-joined description
//! segments tagged with chunk ids when present; otherwise it keeps the
//! description but updates `source_ids` (caller already does that). Full
//! LightRAG cache-replay rebuild can layer on later without changing this API.

use std::collections::HashSet;

use serde_json::{json, Value};

/// Rebuild a description string given remaining chunk/source ids.
///
/// Supported formats (best-effort, DRY with common merge outputs):
/// 1. Segments separated by `\n\n` or ` | ` that contain `[chunk_id=...]`
/// 2. Untagged descriptions → returned unchanged (safe default)
pub fn rebuild_description_from_remaining_sources(
    description: &str,
    remaining_sources: &[String],
) -> String {
    if description.trim().is_empty() || remaining_sources.is_empty() {
        return description.to_string();
    }

    let remaining: HashSet<&str> = remaining_sources.iter().map(String::as_str).collect();

    // Prefer double-newline segments (LLM summary concatenations); a single
    // unsplit string is still one segment so tagged sole claims can clear.
    let join_sep = if description.contains("\n\n") {
        "\n\n"
    } else if description.contains(" | ") {
        " | "
    } else {
        ""
    };
    let segments: Vec<&str> = if join_sep.is_empty() {
        vec![description]
    } else {
        description.split(join_sep).collect()
    };

    let tagged: Vec<&str> = segments
        .iter()
        .copied()
        .filter(|seg| seg.contains("[chunk_id=") || seg.contains("chunk_id="))
        .collect();

    if tagged.is_empty() {
        // No per-chunk tags — cannot surgically rebuild; keep full text.
        return description.to_string();
    }

    let kept: Vec<&str> = tagged
        .into_iter()
        .filter(|seg| {
            remaining.iter().any(|src| {
                seg.contains(&format!("[chunk_id={src}]"))
                    || seg.contains(&format!("chunk_id={src}"))
                    || seg.contains(*src)
            })
        })
        .collect();

    if kept.is_empty() {
        // All tagged segments belonged to deleted docs — clear description
        // rather than leave stale claims (honesty > hallucinated provenance).
        return String::new();
    }

    if join_sep.is_empty() {
        kept[0].to_string()
    } else {
        kept.join(join_sep)
    }
}

<<<<<<< HEAD
=======
/// True when `doc_id` still owns at least one remaining source reference.
fn document_id_still_referenced(doc_id: &str, remaining_sources: &[String]) -> bool {
    let chunk_prefix = format!("{doc_id}-chunk-");
    remaining_sources
        .iter()
        .any(|s| s == doc_id || s.starts_with(&chunk_prefix))
}

/// Document id from a chunk key (`{doc}-chunk-{n}`) or bare doc id.
fn document_id_from_source(source: &str) -> Option<&str> {
    if let Some(idx) = source.find("-chunk-") {
        let doc = &source[..idx];
        if !doc.is_empty() {
            return Some(doc);
        }
    }
    if source.contains("::") || source.is_empty() {
        return None;
    }
    Some(source)
}

fn first_remaining_chunk(remaining_sources: &[String]) -> Option<&str> {
    remaining_sources
        .iter()
        .map(String::as_str)
        .find(|s| s.contains("-chunk-"))
        .or_else(|| remaining_sources.first().map(String::as_str))
}

fn first_remaining_document_id(remaining_sources: &[String]) -> Option<&str> {
    remaining_sources
        .iter()
        .find_map(|s| document_id_from_source(s))
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
/// Apply rebuild to a node/edge properties map (mutates description + source_ids).
pub fn apply_rebuild_to_properties(
    properties: &mut std::collections::HashMap<String, Value>,
    remaining_sources: &[String],
) {
    properties.insert("source_ids".to_string(), json!(remaining_sources));
<<<<<<< HEAD
    properties.remove("source_id");
=======
    // Drop legacy node provenance `source_id`; edge topology is re-injected on upsert.
    if properties
        .get("source_id")
        .and_then(|v| v.as_str())
        .is_none_or(|s| !edgequake_storage::traits::is_topology_entity_ref(s))
    {
        properties.remove("source_id");
    }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

    if let Some(desc) = properties
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::to_string)
    {
        let rebuilt = rebuild_description_from_remaining_sources(&desc, remaining_sources);
        properties.insert("description".to_string(), json!(rebuilt));
    }

    // Keep source_chunk_ids aligned when present
    if properties.contains_key("source_chunk_ids") {
        properties.insert("source_chunk_ids".to_string(), json!(remaining_sources));
    }
<<<<<<< HEAD
=======

    // SPEC-098 Symptom F: clear/rewrite singular citation fields (not GIN SSOT).
    if remaining_sources.is_empty() {
        properties.remove("source_chunk_id");
        properties.remove("source_document_id");
    } else {
        if properties.contains_key("source_chunk_id") {
            match first_remaining_chunk(remaining_sources) {
                Some(chunk) => {
                    properties.insert("source_chunk_id".to_string(), json!(chunk));
                }
                None => {
                    properties.remove("source_chunk_id");
                }
            }
        }
        if properties.contains_key("source_document_id") {
            match first_remaining_document_id(remaining_sources) {
                Some(doc) => {
                    properties.insert("source_document_id".to_string(), json!(doc));
                }
                None => {
                    properties.remove("source_document_id");
                }
            }
        }
    }

    // SPEC-098: keep source_document_ids coherent after shared prune (not GIN SSOT).
    if let Some(Value::Array(docs)) = properties.get("source_document_ids").cloned() {
        let kept: Vec<Value> = docs
            .into_iter()
            .filter(|v| {
                v.as_str()
                    .map(|d| document_id_still_referenced(d, remaining_sources))
                    .unwrap_or(false)
            })
            .collect();
        properties.insert("source_document_ids".to_string(), Value::Array(kept));
    } else if !remaining_sources.is_empty() && properties.contains_key("source_document_id") {
        // No array yet — leave singular as SSOT above.
    }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn keeps_untagged_description() {
        let d = "Alice works at Acme.";
        assert_eq!(
            rebuild_description_from_remaining_sources(d, &["doc1-chunk-0".into()]),
            d
        );
    }

    #[test]
    fn filters_tagged_segments() {
        let d = "From A [chunk_id=doc1-chunk-0]\n\nFrom B [chunk_id=doc2-chunk-0]";
        let out = rebuild_description_from_remaining_sources(d, &["doc1-chunk-0".into()]);
        assert!(out.contains("From A"));
        assert!(!out.contains("From B"));
    }

    #[test]
    fn clears_when_all_tagged_sources_gone() {
        let d = "Only [chunk_id=gone-chunk]";
        let out = rebuild_description_from_remaining_sources(d, &["other".into()]);
        assert!(out.is_empty());
    }

    #[test]
<<<<<<< HEAD
=======
    fn prunes_source_document_ids_to_remaining() {
        let mut props = HashMap::new();
        props.insert("source_ids".into(), json!(["docA-chunk-0", "docB-chunk-0"]));
        props.insert(
            "source_document_ids".into(),
            json!(["docA", "docB", "docC"]),
        );
        apply_rebuild_to_properties(&mut props, &["docB-chunk-0".into()]);
        assert_eq!(props.get("source_ids"), Some(&json!(["docB-chunk-0"])));
        assert_eq!(props.get("source_document_ids"), Some(&json!(["docB"])));
    }

    #[test]
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    fn apply_updates_props() {
        let mut props = HashMap::new();
        props.insert(
            "description".into(),
            json!("Keep [chunk_id=c1]\n\nDrop [chunk_id=c2]"),
        );
        props.insert("source_ids".into(), json!(["c1", "c2"]));
        apply_rebuild_to_properties(&mut props, &["c1".into()]);
        assert_eq!(props["source_ids"], json!(["c1"]));
        let desc = props["description"].as_str().unwrap();
        assert!(desc.contains("Keep"));
        assert!(!desc.contains("Drop"));
    }
<<<<<<< HEAD
=======

    #[test]
    fn clears_singular_citation_fields_when_empty() {
        let mut props = HashMap::new();
        props.insert("source_ids".into(), json!(["docA-chunk-0"]));
        props.insert("source_chunk_id".into(), json!("docA-chunk-0"));
        props.insert("source_document_id".into(), json!("docA"));
        props.insert(
            "source_id".into(),
            json!("ws::ENTITY"), // topology — must survive rebuild
        );
        apply_rebuild_to_properties(&mut props, &[]);
        assert_eq!(props.get("source_ids"), Some(&json!([])));
        assert!(!props.contains_key("source_chunk_id"));
        assert!(!props.contains_key("source_document_id"));
        assert_eq!(props.get("source_id"), Some(&json!("ws::ENTITY")));
    }

    #[test]
    fn rewrites_singulars_to_remaining_shared_sources() {
        let mut props = HashMap::new();
        props.insert("source_ids".into(), json!(["docA-chunk-0", "docB-chunk-1"]));
        props.insert("source_chunk_id".into(), json!("docA-chunk-0"));
        props.insert("source_document_id".into(), json!("docA"));
        apply_rebuild_to_properties(&mut props, &["docB-chunk-1".into()]);
        assert_eq!(props.get("source_chunk_id"), Some(&json!("docB-chunk-1")));
        assert_eq!(props.get("source_document_id"), Some(&json!("docB")));
    }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}
