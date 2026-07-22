//! SSOT for Graph API node `label` (066 Drawing / 067 Opaque ID soft-label / 072).
//!
//! Law: `id` stays the scoped graph node id; `label` is the human surface form.
//! Delegates to [`edgequake_pipeline::resolve_entity_display_label`] so query
//! and lineage share the same presentation rules.

use edgequake_pipeline::resolve_entity_display_label;
use edgequake_storage::traits::GraphNode;

fn prop_str<'a>(node: &'a GraphNode, key: &str) -> Option<&'a str> {
    node.properties.get(key).and_then(|v| v.as_str())
}

/// Human-facing label for a graph node response.
pub fn graph_node_label(node: &GraphNode) -> String {
    resolve_entity_display_label(
        &node.id,
        prop_str(node, "entity_type"),
        prop_str(node, "description"),
        prop_str(node, "display_name"),
        prop_str(node, "label"),
    )
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

    #[test]
    fn opaque_uuid_uses_description_snippet() {
        let n = node(
            "00000000-0000-0000-0000-000000000003::84B69E27-E38B-444A-83DD-5E6A537C6F12",
            &[
                ("entity_type", serde_json::json!("ORGANIZATION")),
                (
                    "description",
                    serde_json::json!("Anthropic API resource referenced in the guide"),
                ),
                (
                    "label",
                    serde_json::json!("84B69E27-E38B-444A-83DD-5E6A537C6F12"),
                ),
            ],
        );
        let label = graph_node_label(&n);
        assert!(label.contains("Anthropic API resource"), "got {label}");
        assert!(
            !label.contains("84B69E27"),
            "must not surface UUID: {label}"
        );
    }

    #[test]
    fn opaque_uuid_without_description_uses_type_badge() {
        let n = node(
            "84b69e27-e38b-444a-83dd-5e6a537c6f12",
            &[("entity_type", serde_json::json!("CONCEPT"))],
        );
        assert_eq!(graph_node_label(&n), "Opaque ID · CONCEPT");
    }
}
