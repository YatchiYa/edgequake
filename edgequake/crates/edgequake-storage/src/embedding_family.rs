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

    /// FK column on the typed embedding table (PK companion to `model_id`).
    pub fn typed_fk_column(self) -> &'static str {
        match self {
            Self::Entity => "entity_id",
            Self::Relationship => "relationship_id",
            Self::Report => "report_id",
        }
    }

    /// Whether the typed FK is UUID (`entity`/`relationship`) or TEXT (`report`).
    pub fn typed_fk_is_uuid(self) -> bool {
        !matches!(self, Self::Report)
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

/// Format `{source}->{target}:{relation_type}` legacy relationship vector id.
///
/// SPEC-130 / LAW-130: SSOT shared by vector batch collect, relational sink
/// report keys, and fleet mirror lookups. Relation type is uppercased via
/// [`crate::normalize_relation_type_str`].
pub fn format_relationship_legacy_key(src: &str, tgt: &str, rel_type: &str) -> String {
    let rt = crate::graph_batch_dedupe::normalize_relation_type_str(rel_type);
    format!("{src}->{tgt}:{rt}")
}

/// Parse `{source}->{target}:{relation_type}` legacy relationship vector id.
///
/// Uses the **last** `->` as the source/target separator so entity names that
/// themselves contain `->` (e.g. LLM-extracted `27_->_25_STRENGTHENING`) still
/// resolve when only the **source** side has arrows. Rel type is taken from the
/// last `:`.
///
/// When the **target** also contains `->`, this naive split is ambiguous —
/// prefer [`parse_relationship_legacy_key_with_resolver`] (SPEC-133) whenever an
/// entity-name existence check is available.
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

/// SPEC-133: disambiguate legacy relationship keys when entity names contain `->`.
///
/// Tries every `->` split of the `source->target` pair. Prefer splits where
/// **both** endpoints satisfy `exists`. Exactly one both-resolve → that split.
/// Multiple both-resolve → rightmost (preserves source-arrow preference when both
/// pairs somehow exist). Zero both-resolve → naive [`parse_relationship_legacy_key`].
pub fn parse_relationship_legacy_key_with_resolver<F>(
    id: &str,
    exists: F,
) -> Option<(String, String, String)>
where
    F: Fn(&str) -> bool,
{
    if id.starts_with("entity:") || id.starts_with("community_report:") {
        return None;
    }
    let (pair, rel_type) = id.rsplit_once(':')?;
    if rel_type.is_empty() {
        return None;
    }

    let mut both_ok: Vec<(String, String)> = Vec::new();
    let bytes = pair.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'>' {
            let source = &pair[..i];
            let target = &pair[i + 2..];
            if !source.is_empty() && !target.is_empty() && exists(source) && exists(target) {
                both_ok.push((source.to_string(), target.to_string()));
            }
            i += 2;
            continue;
        }
        i += 1;
    }

    match both_ok.len() {
        0 => parse_relationship_legacy_key(id),
        n => {
            let (source, target) = both_ok.swap_remove(n - 1);
            Some((source, target, rel_type.to_string()))
        }
    }
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

    /// SPEC-130: format ↔ parse round-trip (uppercase SSOT).
    #[test]
    fn contract_spec130_format_relationship_legacy_key() {
        let key = format_relationship_legacy_key("ALPHA", "BETA", "works_at");
        assert_eq!(key, "ALPHA->BETA:WORKS_AT");
        assert_eq!(
            parse_relationship_legacy_key(&key),
            Some(("ALPHA".into(), "BETA".into(), "WORKS_AT".into()))
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

    /// SPEC-133: target (and multi-arrow) names — naive rsplit invents wrong endpoints.
    #[test]
    fn contract_spec133_parse_target_arrow_naive_is_wrong() {
        let key = format_relationship_legacy_key(
            "FLOW_DIRECTION",
            "ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)",
            "RELATED_TO",
        );
        assert_eq!(
            parse_relationship_legacy_key(&key),
            Some((
                "FLOW_DIRECTION->ARROW_1_(SHADED_BOX_".into(),
                "CIRCULAR_TARGET)".into(),
                "RELATED_TO".into()
            ))
        );
    }

    /// SPEC-133: index-guided parse recovers zz-raw / UI miss keys.
    #[test]
    fn contract_spec133_parse_target_arrow_with_resolver() {
        let cases = [
            (
                "LEFT_MARGIN",
                "LEFT_MARGIN_VALUE_1->_00_->_+",
                "RELATED_TO",
            ),
            (
                "SMALL_BOXED_LABEL_T.",
                "LEFT_MARGIN_LABEL_1->_00_->_+",
                "RELATED_TO",
            ),
            (
                "FLOW_DIRECTION",
                "ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)",
                "RELATED_TO",
            ),
            (
                "FLOW_DIRECTION",
                "ARROW_2_(CIRCULAR_TARGET_->VERTICAL_PANEL)",
                "RELATED_TO",
            ),
            (
                "LEFT_MARGIN_SEQUENCE",
                "SEQUENCE_1->_00_->_+",
                "RELATED_TO",
            ),
        ];
        for (src, tgt, rel) in cases {
            let key = format_relationship_legacy_key(src, tgt, rel);
            let parsed = parse_relationship_legacy_key_with_resolver(&key, |n| {
                n == src || n == tgt
            });
            assert_eq!(
                parsed,
                Some((src.into(), tgt.into(), rel.into())),
                "key={key}"
            );
        }
    }

    /// SPEC-133 / LAW-133-7: source-arrow still unique both-resolve under resolver.
    #[test]
    fn contract_spec133_parse_source_arrow_with_resolver() {
        let miss = "27_->_25_STRENGTHENING->CLAIM_FRONTIER:STRENGTHENS";
        let parsed = parse_relationship_legacy_key_with_resolver(miss, |n| {
            n == "27_->_25_STRENGTHENING" || n == "CLAIM_FRONTIER"
        });
        assert_eq!(
            parsed,
            Some((
                "27_->_25_STRENGTHENING".into(),
                "CLAIM_FRONTIER".into(),
                "STRENGTHENS".into()
            ))
        );
    }

    /// SPEC-133: empty existence check falls back to naive rsplit.
    #[test]
    fn contract_spec133_parse_resolver_empty_falls_back() {
        let key = format_relationship_legacy_key(
            "FLOW_DIRECTION",
            "ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)",
            "RELATED_TO",
        );
        let parsed = parse_relationship_legacy_key_with_resolver(&key, |_| false);
        assert_eq!(parsed, parse_relationship_legacy_key(&key));
    }

    #[test]
    fn contract_iw2_parse_relationship_key_rejects_empty_sides() {
        assert!(parse_relationship_legacy_key("->B:TYPE").is_none());
        assert!(parse_relationship_legacy_key("A->:TYPE").is_none());
        assert!(parse_relationship_legacy_key("A->B:").is_none());
        assert!(parse_relationship_legacy_key("entity:A->B:TYPE").is_none());
    }

    /// SPEC-120: absorb SQL is driven by family metadata (SOLID OCP / DRY).
    #[test]
    fn contract_spec120_family_typed_fk_metadata() {
        assert_eq!(EmbeddingFamily::Entity.typed_table(), "entity_embeddings");
        assert_eq!(EmbeddingFamily::Entity.typed_fk_column(), "entity_id");
        assert!(EmbeddingFamily::Entity.typed_fk_is_uuid());

        assert_eq!(
            EmbeddingFamily::Relationship.typed_table(),
            "relationship_embeddings"
        );
        assert_eq!(
            EmbeddingFamily::Relationship.typed_fk_column(),
            "relationship_id"
        );
        assert!(EmbeddingFamily::Relationship.typed_fk_is_uuid());

        assert_eq!(EmbeddingFamily::Report.typed_table(), "report_embeddings");
        assert_eq!(EmbeddingFamily::Report.typed_fk_column(), "report_id");
        assert!(!EmbeddingFamily::Report.typed_fk_is_uuid());
    }
}
