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

/// Apply rebuild to a node/edge properties map (mutates description + source_ids).
pub fn apply_rebuild_to_properties(
    properties: &mut std::collections::HashMap<String, Value>,
    remaining_sources: &[String],
) {
    properties.insert("source_ids".to_string(), json!(remaining_sources));
    properties.remove("source_id");

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
}
