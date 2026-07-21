//! Entity identity newtype and the single normalization entry point.
//!
//! # WHY THIS EXISTS (RC-6 / P-G1)
//!
//! Before this module, entity identity was a *convention* enforced nowhere:
//! - The orchestrator merger wrote graph nodes as `JOHN_DOE` (normalized) and
//!   entity vectors as the bare `JOHN_DOE`.
//! - The async processor wrote graph nodes as `John Doe` (raw) and entity
//!   vectors as `entity:John Doe` (raw, prefixed).
//! - The sync upload path wrote graph nodes as `JOHN_DOE` and vectors as
//!   `entity:JOHN_DOE`.
//!
//! Three writers, three conventions. The result was a silently fragmented
//! knowledge graph: the same real entity became multiple nodes, and the entity
//! vectors written by the processor were invisible to the query layer (which
//! looked them up by the graph node id `JOHN_DOE`).
//!
//! First principle: **identity is a value, not a convention.** An `EntityId` is
//! a normalized newtype. The graph node id and the entity vector id are both
//! *derived* from it, so they can never diverge by construction. No writer can
//! build an un-normalized entity id.
//!
//! # Canonical convention
//!
//! - Graph node id (legacy / no workspace) = `EntityId::as_graph_node_id()` → bare `JOHN_DOE`
//! - Graph node id (workspace-scoped, SPEC-032 / B3b) =
//!   `EntityId::scoped_graph_node_id(workspace_id)` → `{workspace_id}::JOHN_DOE`
//! - Entity vector id = `EntityId::as_vector_id()` → `entity:JOHN_DOE`
//!
//! WHY scoped graph ids: a shared AGE graph with bare `node_id` + UNIQUE
//! `eq_node_id` lets the first workspace to extract `SURGERY` own that vertex;
//! later Acc workspaces merge into foreign `workspace_id` rows and Mix query
//! (workspace filter) cannot see them. Vectors stay workspace-table-scoped, so
//! extract density looks healthy while the graph arm is starved.
//!
//! The `entity:` prefix on the vector id is what [`VectorId`] decodes back into
//! an [`EntityId`]; keeping the prefix makes the storage id self-describing
//! even when metadata is absent. Display `label` stays the bare normalized name.

//!
//! [`VectorId`]: crate::vector_id::VectorId

use crate::vector_id::VectorId;

/// A normalized entity identity.
///
/// Constructed exclusively via [`EntityId::new`], which runs the single
/// canonical normalizer. The wrapped string is always normalized
/// (UPPERCASE_UNDERSCORE, prefixes/possessives stripped). The empty string is
/// a valid interior value only when the input was empty/whitespace; callers
/// should check [`EntityId::is_empty`] and skip the write in that case (E1).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(String);

impl EntityId {
    /// Construct a normalized entity id from any raw name.
    ///
    /// Defensively strips a leading `entity:` prefix if a caller accidentally
    /// passes an already-prefixed value (E2), so `EntityId::new("entity:Foo")`
    /// and `EntityId::new("Foo")` produce the same identity.
    pub fn new(raw: &str) -> Self {
        let stripped = raw.strip_prefix("entity:").unwrap_or(raw);
        Self(normalize_entity_name(stripped))
    }

    /// Construct from an already-normalized string, bypassing normalization.
    ///
    /// This is the mirror of [`as_str`](EntityId::as_str) and exists so trusted
    /// readers (e.g. reconstructing an id from a graph node) can avoid
    /// re-normalizing. The caller guarantees the input is already normalized.
    pub fn from_normalized(normalized: impl Into<String>) -> Self {
        Self(normalized.into())
    }

    /// Separator between workspace UUID and normalized entity name in scoped
    /// graph node ids. Chosen so it cannot appear in UUIDs and is distinct from
    /// relationship `A::B` vector ids (those use a single `::` between *two
    /// entity names*, not a workspace prefix).
    pub const WORKSPACE_SCOPE_SEP: &str = "::";

    /// The bare normalized name (legacy graph node id when no workspace).
    pub fn as_graph_node_id(&self) -> &str {
        &self.0
    }

    /// Workspace-scoped graph node id: `{workspace_id}::{NORMALIZED_NAME}`.
    ///
    /// Empty / whitespace workspace falls back to the bare id (single-tenant /
    /// tests without a workspace context).
    pub fn scoped_graph_node_id(&self, workspace_id: &str) -> String {
        let ws = workspace_id.trim();
        if ws.is_empty() || self.0.is_empty() {
            return self.0.clone();
        }
        format!("{ws}{}{}", Self::WORKSPACE_SCOPE_SEP, self.0)
    }

    /// Resolve graph node id: scoped when `workspace_id` is `Some` and non-empty.
    pub fn graph_node_id_for_workspace(&self, workspace_id: Option<&str>) -> String {
        match workspace_id.map(str::trim).filter(|s| !s.is_empty()) {
            Some(ws) => self.scoped_graph_node_id(ws),
            None => self.0.clone(),
        }
    }

    /// Strip a leading `{workspace_id}::` scope when present; otherwise return
    /// the bare normalized name (for display / keyword match).
    pub fn bare_name_from_graph_node_id(graph_node_id: &str) -> &str {
        match graph_node_id.split_once(Self::WORKSPACE_SCOPE_SEP) {
            Some((maybe_ws, rest))
                if !rest.is_empty()
                    && maybe_ws.len() == 36
                    && maybe_ws.chars().filter(|c| *c == '-').count() == 4 =>
            {
                rest
            }
            _ => graph_node_id,
        }
    }

    /// The prefixed vector storage id (`entity:NAME`), used as the entity
    /// vector id.
    pub fn as_vector_id(&self) -> String {
        format!("entity:{}", self.0)
    }

    /// The bare normalized name as a borrowed string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True if the normalized name is empty (E1). Callers should skip writes
    /// for empty ids and log a warning.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Decode an entity vector storage id (e.g. `entity:JOHN_DOE`) back into an
    /// `EntityId`. Returns `None` for non-entity vector ids (chunk/relationship)
    /// or for an empty entity name.
    pub fn from_vector_storage_id(storage_id: &str) -> Option<Self> {
        VectorId::from_storage_id(storage_id).and_then(|vid| match vid {
            VectorId::Entity { name } => {
                let name = name.strip_prefix("entity:").unwrap_or(&name);
                if name.is_empty() {
                    None
                } else {
                    Some(Self::from_normalized(name))
                }
            }
            _ => None,
        })
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&EntityId> for String {
    fn from(id: &EntityId) -> Self {
        id.0.clone()
    }
}

/// The single canonical entity-name normalizer.
///
/// This is the **only** place entity names are normalized in the codebase.
/// `edgequake-pipeline` re-exports this function as
/// `edgequake_pipeline::prompts::normalize_entity_name` for backwards
/// compatibility; do not duplicate the logic elsewhere (DRY).
///
/// Transformations:
/// - Trims surrounding whitespace.
/// - Strips common leading articles ("The", "A", "An" in any case).
/// - Strips possessive suffixes (`'s`) per word.
/// - Title-cases each word, joins with `_`, uppercases the result.
///
/// Empty / whitespace-only input yields the empty string (E1).
///
/// Also rejects LightRAG `normalize_extracted_info` numeric empties (056):
/// pure digits with `len < 3`, or digits+dots with `len < 6` and at least one dot.
pub fn normalize_entity_name(raw_name: &str) -> String {
    let trimmed = raw_name.trim();
    if trimmed.is_empty() || is_lightrag_rejected_numeric_name(trimmed) {
        return String::new();
    }

    let without_prefix = trimmed
        .strip_prefix("The ")
        .or_else(|| trimmed.strip_prefix("the "))
        .or_else(|| trimmed.strip_prefix("A "))
        .or_else(|| trimmed.strip_prefix("a "))
        .or_else(|| trimmed.strip_prefix("An "))
        .or_else(|| trimmed.strip_prefix("an "))
        .unwrap_or(trimmed);

    let normalized = without_prefix
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|word| {
            let without_possessive = word
                .strip_suffix("'s")
                .or_else(|| word.strip_suffix("'s"))
                .unwrap_or(word);
            to_title_case(without_possessive)
        })
        .collect::<Vec<_>>()
        .join("_")
        .to_uppercase();

    // Re-check after fold (e.g. dotted forms that survived whitespace split).
    if is_lightrag_rejected_numeric_name(&normalized) {
        return String::new();
    }
    normalized
}

/// LightRAG `normalize_extracted_info` empty-name filters for short numeric labels.
fn is_lightrag_rejected_numeric_name(name: &str) -> bool {
    let t = name.trim();
    if t.is_empty() {
        return false;
    }
    if t.len() < 3 && t.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let digits_and_dots = t.chars().all(|c| c.is_ascii_digit() || c == '.');
    if t.len() < 6 && digits_and_dots && t.contains('.') {
        return true;
    }
    false
}

/// Convert a word to title case (first letter uppercase, rest lowercase).
fn to_title_case(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first
            .to_uppercase()
            .chain(chars.flat_map(|c| c.to_lowercase()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_normalizes_casing_variants_to_one_identity() {
        // P-G1 acceptance (unit level): three casing variants → one EntityId.
        let a = EntityId::new("John Doe");
        let b = EntityId::new("john doe");
        let c = EntityId::new("JOHN DOE");
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a.as_graph_node_id(), "JOHN_DOE");
    }

    #[test]
    fn graph_node_id_and_vector_id_are_derived_consistently() {
        let id = EntityId::new("Sarah Chen");
        assert_eq!(id.as_graph_node_id(), "SARAH_CHEN");
        assert_eq!(id.as_vector_id(), "entity:SARAH_CHEN");
    }

    #[test]
    fn vector_id_round_trips_through_storage_id() {
        // P-G1: EntityId → vector id → from_storage_id → EntityId.
        let id = EntityId::new("Apple Inc");
        let vid = id.as_vector_id();
        let decoded = EntityId::from_vector_storage_id(&vid).unwrap();
        assert_eq!(decoded, id);
    }

    #[test]
    fn strips_accidental_entity_prefix() {
        // E2: a caller passing an already-prefixed value must not double-prefix.
        assert_eq!(EntityId::new("entity:Foo Bar"), EntityId::new("Foo Bar"));
        assert_eq!(
            EntityId::new("entity:Foo Bar").as_vector_id(),
            "entity:FOO_BAR"
        );
    }

    #[test]
    fn empty_name_is_empty_identity() {
        // E1.
        assert!(EntityId::new("").is_empty());
        assert!(EntityId::new("   ").is_empty());
        assert_eq!(EntityId::new("").as_graph_node_id(), "");
    }

    #[test]
    fn non_entity_storage_id_decodes_to_none() {
        assert!(EntityId::from_vector_storage_id("doc-123-chunk-0").is_none());
        assert!(EntityId::from_vector_storage_id("A::B").is_none());
    }

    #[test]
    fn prefixes_and_possessives_stripped() {
        assert_eq!(EntityId::new("The Company").as_str(), "COMPANY");
        assert_eq!(EntityId::new("John's").as_str(), "JOHN");
    }

    #[test]
    fn non_ascii_preserved_and_normalized() {
        // E3: non-ASCII names are handled by the existing title-case logic.
        assert_eq!(EntityId::new("René Descartes").as_str(), "RENÉ_DESCARTES");
    }

    #[test]
    fn hyphens_and_special_chars_preserved() {
        assert_eq!(EntityId::new("New-York").as_str(), "NEW-YORK");
        assert_eq!(EntityId::new("C++").as_str(), "C++");
    }

    #[test]
    fn scoped_graph_node_id_prefixes_workspace() {
        let id = EntityId::new("Basal Cell");
        let ws = "e0270f5f-0b6c-4e90-882f-5f9b0eac8cff";
        assert_eq!(id.scoped_graph_node_id(ws), format!("{ws}::BASAL_CELL"));
        assert_eq!(
            id.graph_node_id_for_workspace(Some(ws)),
            id.scoped_graph_node_id(ws)
        );
        assert_eq!(id.graph_node_id_for_workspace(None), "BASAL_CELL");
        assert_eq!(id.scoped_graph_node_id("  "), "BASAL_CELL");
    }

    #[test]
    fn bare_name_strips_uuid_workspace_scope() {
        let ws = "e0270f5f-0b6c-4e90-882f-5f9b0eac8cff";
        let scoped = format!("{ws}::BASAL_CELL");
        assert_eq!(
            EntityId::bare_name_from_graph_node_id(&scoped),
            "BASAL_CELL"
        );
        assert_eq!(
            EntityId::bare_name_from_graph_node_id("BASAL_CELL"),
            "BASAL_CELL"
        );
        // Relationship-style A::B must not be stripped (left side is not a UUID).
        assert_eq!(
            EntityId::bare_name_from_graph_node_id("ALPHA::BETA"),
            "ALPHA::BETA"
        );
    }

    #[test]
    fn lightrag_short_numeric_names_normalize_empty() {
        // 056 / LightRAG normalize_extracted_info
        assert_eq!(normalize_entity_name("42"), "");
        assert_eq!(normalize_entity_name("7"), "");
        assert_eq!(normalize_entity_name("1.2"), "");
        assert_eq!(normalize_entity_name("12.3"), "");
        // Kept: years and real names
        assert_eq!(normalize_entity_name("2022"), "2022");
        assert_eq!(normalize_entity_name("BRCA1"), "BRCA1");
        assert_eq!(normalize_entity_name("5 Fluorouracil"), "5_FLUOROURACIL");
    }
}
