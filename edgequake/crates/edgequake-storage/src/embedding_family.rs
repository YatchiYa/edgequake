//! SPEC-091 IW2 — embedding family taxonomy for typed fleet cutover.
//!
//! Single source of truth for legacy `eq_*_vectors` id shapes and typed table
//! names. Shared by the engine backfill, dual-write writer, and typed read path.

/// Typed embedding family (chunk handled separately by W3 machinery).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddingFamily {
    Entity,
    Relationship,
    Report,
}

impl EmbeddingFamily {
    pub fn typed_table(self) -> &'static str {
        match self {
            Self::Entity => "entity_embeddings",
            Self::Relationship => "relationship_embeddings",
            Self::Report => "report_embeddings",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relationship => "relationship",
            Self::Report => "report",
        }
    }

    /// Engine backfill step suffix (cursor `family` field).
    pub fn backfill_family_key(self) -> &'static str {
        self.as_str()
    }

    pub const FLEET_BACKFILL_FAMILIES: [Self; 3] = [Self::Entity, Self::Relationship, Self::Report];
}

/// Classify a legacy vector storage id (non-chunk families only).
pub fn classify_legacy_vector_id(id: &str) -> Option<EmbeddingFamily> {
    if id.contains("-chunk-") {
        return None;
    }
    if id.starts_with("community_report:") {
        return Some(EmbeddingFamily::Report);
    }
    if id.starts_with("entity:") {
        return Some(EmbeddingFamily::Entity);
    }
    if parse_relationship_legacy_key(id).is_some() {
        return Some(EmbeddingFamily::Relationship);
    }
    None
}

/// Strip the `entity:` prefix from a legacy entity vector id.
pub fn entity_name_from_legacy_id(id: &str) -> Option<&str> {
    id.strip_prefix("entity:").filter(|s| !s.is_empty())
}

/// Parse `{source}->{target}:{relation_type}` legacy relationship vector id.
///
/// Uses the **last** `->` as the source/target separator so entity names that
/// themselves contain `->` (e.g. LLM-extracted `27_->_25_STRENGTHENING`) still
/// resolve. Rel type is taken from the last `:`. Residual ambiguity remains if
/// the **target** name also contains `->`.
pub fn parse_relationship_legacy_key(id: &str) -> Option<(String, String, String)> {
    if id.starts_with("entity:") || id.starts_with("community_report:") {
        return None;
    }
    let (pair, rel_type) = id.rsplit_once(':')?;
    if rel_type.is_empty() {
        return None;
    }
    let (source, target) = pair.rsplit_once("->")?;
    if source.is_empty() || target.is_empty() {
        return None;
    }
    Some((source.to_string(), target.to_string(), rel_type.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_iw2_classify_entity() {
        assert_eq!(
            classify_legacy_vector_id("entity:SARAH_CHEN"),
            Some(EmbeddingFamily::Entity)
        );
    }

    #[test]
    fn contract_iw2_classify_relationship() {
        assert_eq!(
            classify_legacy_vector_id("ALPHA->BETA:WORKS_AT"),
            Some(EmbeddingFamily::Relationship)
        );
    }

    #[test]
    fn contract_iw2_classify_report() {
        assert_eq!(
            classify_legacy_vector_id("community_report:3"),
            Some(EmbeddingFamily::Report)
        );
    }

    #[test]
    fn contract_iw2_parse_relationship_key() {
        assert_eq!(
            parse_relationship_legacy_key("A->B:TYPE"),
            Some(("A".into(), "B".into(), "TYPE".into()))
        );
    }

    /// SPEC-098 / Argus miss class: source entity name contains `->`.
    #[test]
    fn contract_iw2_parse_relationship_key_arrow_in_source() {
        let miss = "27_->_25_STRENGTHENING->CLAIM_FRONTIER:STRENGTHENS";
        assert_eq!(
            parse_relationship_legacy_key(miss),
            Some((
                "27_->_25_STRENGTHENING".into(),
                "CLAIM_FRONTIER".into(),
                "STRENGTHENS".into()
            ))
        );
        assert_eq!(
            classify_legacy_vector_id(miss),
            Some(EmbeddingFamily::Relationship)
        );
    }

    #[test]
    fn contract_iw2_parse_relationship_key_rejects_empty_sides() {
        assert!(parse_relationship_legacy_key("->B:TYPE").is_none());
        assert!(parse_relationship_legacy_key("A->:TYPE").is_none());
        assert!(parse_relationship_legacy_key("A->B:").is_none());
        assert!(parse_relationship_legacy_key("entity:A->B:TYPE").is_none());
    }
}
