//! Human presentation labels for graph entities (066 / 067 / 072).
//!
//! Law: `id` stays the graph node id; presentation never surfaces bare opaque
//! machine identifiers. Multimodal Drawing/Table/Equation use the 066 mm path.

use edgequake_storage::is_opaque_identifier;

use crate::multimodal::{bare_entity_id, resolve_mm_display_from_node_props};

/// Soft human label for legacy opaque-id nodes (067). Never invents a proper name.
pub fn soft_label_opaque(entity_type: Option<&str>, description: Option<&str>) -> String {
    if let Some(desc) = description.map(str::trim).filter(|s| !s.is_empty()) {
        let mut out = String::new();
        for (i, ch) in desc.chars().enumerate() {
            if i >= 60 {
                out.push('…');
                break;
            }
            out.push(ch);
        }
        return out;
    }
    let ty = entity_type
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("Entity");
    format!("Opaque ID · {ty}")
}

/// Human-facing label from node identity + properties (SSOT for API + query).
///
/// Prefer `display_name`, then non-opaque stored `label`, then soft-label when
/// the bare id is opaque, else the bare semantic name.
pub fn resolve_entity_display_label(
    node_id: &str,
    entity_type: Option<&str>,
    description: Option<&str>,
    display_name: Option<&str>,
    stored_label: Option<&str>,
) -> String {
    let is_mm = matches!(
        entity_type.map(|s| s.to_ascii_lowercase()).as_deref(),
        Some("drawing" | "table" | "equation")
    );

    if is_mm {
        return resolve_mm_display_from_node_props(
            node_id,
            description,
            entity_type,
            display_name.or(stored_label),
        );
    }

    if let Some(d) = display_name.map(str::trim).filter(|s| !s.is_empty()) {
        if !is_opaque_identifier(d) {
            return d.to_string();
        }
    }
    if let Some(l) = stored_label.map(str::trim).filter(|s| !s.is_empty()) {
        if l != node_id && !is_opaque_identifier(l) {
            return l.to_string();
        }
    }

    let bare = bare_entity_id(node_id);
    if is_opaque_identifier(bare) {
        return soft_label_opaque(entity_type, description);
    }
    bare.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_uuid_uses_description_snippet() {
        let label = resolve_entity_display_label(
            "00000000-0000-0000-0000-000000000003::84B69E27-E38B-444A-83DD-5E6A537C6F12",
            Some("ORGANIZATION"),
            Some("Anthropic API resource referenced in the guide"),
            None,
            Some("84B69E27-E38B-444A-83DD-5E6A537C6F12"),
        );
        assert!(label.contains("Anthropic API resource"), "got {label}");
        assert!(
            !label.contains("84B69E27"),
            "must not surface UUID: {label}"
        );
    }

    #[test]
    fn opaque_uuid_without_description_uses_type_badge() {
        let label = resolve_entity_display_label(
            "84b69e27-e38b-444a-83dd-5e6a537c6f12",
            Some("CONCEPT"),
            None,
            None,
            None,
        );
        assert_eq!(label, "Opaque ID · CONCEPT");
    }

    #[test]
    fn person_uses_bare_label() {
        let label = resolve_entity_display_label(
            "00000000-0000-0000-0000-000000000003::SARAH_CHEN",
            Some("PERSON"),
            None,
            None,
            Some("SARAH_CHEN"),
        );
        assert_eq!(label, "SARAH_CHEN");
    }
}
