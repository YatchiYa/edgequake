//! SSOT for Graph API node `label` (066 Drawing Entity Naming).
//!
//! Law: `id` stays the scoped graph node id; `label` is the human surface form.
//! Prefer `properties.display_name`, then lazy resolve for mm types from
//! description + id, then bare `properties.label`, then node id.

use edgequake_pipeline::resolve_mm_display_from_node_props;
use edgequake_storage::traits::GraphNode;

fn prop_str<'a>(node: &'a GraphNode, key: &str) -> Option<&'a str> {
    node.properties.get(key).and_then(|v| v.as_str())
}

/// Human-facing label for a graph node response.
pub fn graph_node_label(node: &GraphNode) -> String {
    let entity_type = prop_str(node, "entity_type");
    let description = prop_str(node, "description");
    let display_name = prop_str(node, "display_name");
    let stored_label = prop_str(node, "label");

    let is_mm = matches!(
        entity_type.map(|s| s.to_ascii_lowercase()).as_deref(),
        Some("drawing" | "table" | "equation")
    );

    if is_mm {
        return resolve_mm_display_from_node_props(
            &node.id,
            description,
            entity_type,
            display_name.or(stored_label),
        );
    }

    // Non-mm: prefer explicit display_name, then properties.label if it differs
    // usefully from the scoped id, else bare id without workspace scope.
    if let Some(d) = display_name.map(str::trim).filter(|s| !s.is_empty()) {
        return d.to_string();
    }
    if let Some(l) = stored_label.map(str::trim).filter(|s| !s.is_empty()) {
        if l != node.id.as_str() {
            return l.to_string();
        }
    }
    edgequake_pipeline::bare_entity_id(&node.id).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn node(id: &str, props: &[(&str, serde_json::Value)]) -> GraphNode {
        let mut properties = HashMap::new();
        for (k, v) in props {
            properties.insert((*k).to_string(), v.clone());
        }
        GraphNode {
            id: id.to_string(),
            properties,
        }
    }

    #[test]
    fn drawing_uses_display_name_prop() {
        let n = node(
            "ws::IM-PAGE-0002-FIG-01",
            &[
                ("entity_type", serde_json::json!("drawing")),
                (
                    "display_name",
                    serde_json::json!("Architecture overview · p.2 · Fig 1"),
                ),
                ("label", serde_json::json!("IM-PAGE-0002-FIG-01")),
            ],
        );
        assert_eq!(graph_node_label(&n), "Architecture overview · p.2 · Fig 1");
    }

    #[test]
    fn drawing_lazy_resolves_from_description() {
        let n = node(
            "IM-DOC-PAGE-0003-FIG-02",
            &[
                ("entity_type", serde_json::json!("drawing")),
                (
                    "description",
                    serde_json::json!("[Figure Name]Revenue chart\n[Image Type]Chart\n\nbody"),
                ),
                ("label", serde_json::json!("IM-DOC-PAGE-0003-FIG-02")),
            ],
        );
        let label = graph_node_label(&n);
        assert!(label.contains("Revenue chart"), "got {label}");
        assert!(label.contains("p.3"), "got {label}");
    }

    #[test]
    fn person_uses_bare_label() {
        let n = node(
            "00000000-0000-0000-0000-000000000003::SARAH_CHEN",
            &[
                ("entity_type", serde_json::json!("PERSON")),
                ("label", serde_json::json!("SARAH_CHEN")),
            ],
        );
        assert_eq!(graph_node_label(&n), "SARAH_CHEN");
    }
}
