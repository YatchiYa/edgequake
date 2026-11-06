//! Ontology compiled for extraction: prompt sections, type mapping, name canonicalization.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use aho_corasick::{AhoCorasick, MatchKind};
use indexmap::IndexMap;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::model::{validate, ValidationMode, WorkspaceOntology, METADATA_KEY};
use crate::extractor::ExtractionResult;
use crate::prompts::{normalize_entity_name, normalize_type_token};

/// Characters of each definition shown in the prompt.
const PROMPT_DESCRIPTION_CHARS: usize = 240;
/// Budget of the static ontology section of the system prompt.
const SYSTEM_SECTION_CHARS: usize = 8_000;
/// Reference terms listed for one chunk.
const MAX_TERM_HINTS: usize = 40;

#[derive(Debug, Clone)]
struct Term {
    /// Canonical graph key (`normalize_entity_name` of the term name).
    key: String,
    display: String,
    entity_type: Option<String>,
    aliases: Vec<String>,
}

/// A validated ontology ready for the extraction hot path.
pub struct CompiledOntology {
    fingerprint: String,
    name: String,
    system_section: String,
    /// Type id or synonym (normalized) → type id.
    type_map: HashMap<String, String>,
    descriptions: HashMap<String, String>,
    terms: Vec<Term>,
    /// Canonical key or alias key → term index.
    by_key: HashMap<String, usize>,
    matcher: Option<AhoCorasick>,
    /// Matcher pattern index → term index.
    pattern_term: Vec<usize>,
}

impl fmt::Debug for CompiledOntology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompiledOntology")
            .field("name", &self.name)
            .field("fingerprint", &self.fingerprint)
            .field("types", &self.descriptions.len())
            .field("terms", &self.terms.len())
            .finish()
    }
}

// Schemas compare by content: two compilations of the same document are equal.
impl PartialEq for CompiledOntology {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}
impl Eq for CompiledOntology {}

/// What [`CompiledOntology::canonicalize`] changed in one extraction result.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CanonicalizeStats {
    pub renamed: usize,
    pub retyped: usize,
    pub merged: usize,
    pub self_loops_dropped: usize,
}

fn short(text: &str) -> String {
    let t = text.trim();
    if t.chars().count() <= PROMPT_DESCRIPTION_CHARS {
        t.to_string()
    } else {
        format!(
            "{}…",
            t.chars().take(PROMPT_DESCRIPTION_CHARS).collect::<String>()
        )
    }
}

/// Lowercase with every non-alphanumeric character as a space, padded with spaces, so
/// a pattern ` leap 1a ` only matches whole words.
fn fold(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push(' ');
    let mut last_space = true;
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            out.push(c);
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    if !last_space {
        out.push(' ');
    }
    out
}

impl CompiledOntology {
    /// Compile an ontology that already passed [`validate`].
    pub fn compile(ontology: &WorkspaceOntology) -> Self {
        let fingerprint = {
            let bytes = serde_json::to_vec(ontology).unwrap_or_default();
            format!("{:x}", Sha256::digest(bytes))
        };
        let mut type_map = HashMap::new();
        let mut descriptions = HashMap::new();
        for t in &ontology.entity_types {
            type_map.insert(t.id.clone(), t.id.clone());
            for s in &t.synonyms {
                type_map.insert(s.clone(), t.id.clone());
            }
            descriptions.insert(t.id.clone(), t.description.clone());
        }

        let mut terms = Vec::with_capacity(ontology.terms.len());
        let mut by_key = HashMap::new();
        let mut patterns: Vec<String> = Vec::new();
        let mut pattern_term = Vec::new();
        for (i, t) in ontology.terms.iter().enumerate() {
            let key = normalize_entity_name(&t.name);
            by_key.insert(key.clone(), i);
            for form in std::iter::once(&t.name).chain(t.aliases.iter()) {
                by_key.entry(normalize_entity_name(form)).or_insert(i);
                let folded = fold(form);
                if folded.trim().chars().count() >= 2 {
                    patterns.push(folded);
                    pattern_term.push(i);
                }
            }
            terms.push(Term {
                key,
                display: t.name.clone(),
                entity_type: t.entity_type.clone(),
                aliases: t.aliases.clone(),
            });
        }
        let matcher = (!patterns.is_empty())
            .then(|| {
                AhoCorasick::builder()
                    .match_kind(MatchKind::LeftmostLongest)
                    .build(&patterns)
                    .ok()
            })
            .flatten();

        Self {
            fingerprint,
            name: ontology.name.clone(),
            system_section: system_section(ontology),
            type_map,
            descriptions,
            terms,
            by_key,
            matcher,
            pattern_term,
        }
    }

    /// Read `metadata.ontology`. Absent → `None`. Unreadable or invalid → `None` with a
    /// warning: a stored ontology never stops ingestion (LAW-184-2).
    pub fn from_workspace_metadata(metadata: &HashMap<String, Value>) -> Option<Arc<Self>> {
        let raw = metadata.get(METADATA_KEY)?;
        if raw.is_null() {
            return None;
        }
        let parsed: WorkspaceOntology = match serde_json::from_value(raw.clone()) {
            Ok(o) => o,
            Err(error) => {
                tracing::warn!(%error, "Workspace ontology is not readable; extraction ignores it");
                return None;
            }
        };
        match validate(parsed, ValidationMode::Repair) {
            Ok(v)
                if v.ontology.entity_types.is_empty()
                    && v.ontology.terms.is_empty()
                    && v.ontology.relation_types.is_empty() =>
            {
                None
            }
            Ok(v) => Some(Arc::new(Self::compile(&v.ontology))),
            Err(errors) => {
                tracing::warn!(
                    ?errors,
                    "Workspace ontology is invalid; extraction ignores it"
                );
                None
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Ontology type id for a raw type written with an id or a synonym.
    pub fn canonical_type(&self, raw_type: &str) -> Option<&str> {
        self.type_map
            .get(&normalize_type_token(raw_type))
            .map(String::as_str)
    }

    /// The ontology's definition of a type, when it has one.
    pub fn type_description(&self, type_id: &str) -> Option<&str> {
        self.descriptions
            .get(type_id)
            .map(String::as_str)
            .filter(|d| !d.trim().is_empty())
    }

    /// Canonical graph key and type of a name that is a term or one of its aliases.
    pub fn resolve_name(&self, name: &str) -> Option<(&str, Option<&str>)> {
        let term = &self.terms[*self.by_key.get(&normalize_entity_name(name))?];
        Some((term.key.as_str(), term.entity_type.as_deref()))
    }

    /// Every key a term may already have in the graph (canonical and alias keys).
    pub fn graph_keys(&self) -> impl Iterator<Item = (&str, usize)> {
        self.by_key.iter().map(|(k, &i)| (k.as_str(), i))
    }

    /// Canonical key, display name and type of term `i`.
    pub fn term(&self, i: usize) -> (&str, &str, Option<&str>) {
        let t = &self.terms[i];
        (t.key.as_str(), t.display.as_str(), t.entity_type.as_deref())
    }

    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// Static section of the system prompt (types, hierarchy, relations, naming rule).
    pub fn system_prompt_section(&self) -> &str {
        &self.system_section
    }

    /// Reference terms mentioned in `text`, as a prompt section; empty when none.
    pub fn term_hints_section(&self, text: &str) -> String {
        let Some(matcher) = &self.matcher else {
            return String::new();
        };
        let folded = fold(text);
        let mut seen: Vec<usize> = Vec::new();
        for m in matcher.find_iter(&folded) {
            let term = self.pattern_term[m.pattern().as_usize()];
            if !seen.contains(&term) {
                seen.push(term);
                if seen.len() == MAX_TERM_HINTS {
                    break;
                }
            }
        }
        if seen.is_empty() {
            return String::new();
        }
        let mut out = String::from(
            "## Reference Terms In This Text\nThe text mentions these reference terms. Use exactly the canonical name and type given here, even when the text uses another spelling:\n",
        );
        for i in seen {
            let t = &self.terms[i];
            out.push_str(&format!("- \"{}\"", t.display));
            if let Some(ty) = &t.entity_type {
                out.push_str(&format!(" ({ty})"));
            }
            if !t.aliases.is_empty() {
                out.push_str(&format!(" — also written: {}", t.aliases.join(", ")));
            }
            out.push('\n');
        }
        out.push('\n');
        out
    }

    /// Map types written with synonyms to their id; rename aliases to the canonical term,
    /// give terms their type, then fold the duplicates this creates.
    pub fn canonicalize(&self, result: &mut ExtractionResult) -> CanonicalizeStats {
        let mut stats = CanonicalizeStats::default();
        let mut merged: IndexMap<String, crate::extractor::ExtractedEntity> = IndexMap::new();
        for mut entity in std::mem::take(&mut result.entities) {
            if let Some(id) = self.canonical_type(&entity.entity_type) {
                if id != entity.entity_type {
                    entity.entity_type = id.to_string();
                    stats.retyped += 1;
                }
            }
            if let Some((key, ty)) = self.resolve_name(&entity.name) {
                if key != entity.name {
                    entity.name = key.to_string();
                    stats.renamed += 1;
                }
                if let Some(ty) = ty {
                    if ty != entity.entity_type {
                        entity.entity_type = ty.to_string();
                        stats.retyped += 1;
                    }
                }
            }
            match merged.get_mut(&entity.name) {
                Some(kept) => {
                    stats.merged += 1;
                    let extra = entity.description.trim();
                    if !extra.is_empty() && !kept.description.contains(extra) {
                        if !kept.description.is_empty() {
                            kept.description.push('\n');
                        }
                        kept.description.push_str(extra);
                    }
                    kept.importance = kept.importance.max(entity.importance);
                    for span in entity.source_spans {
                        if !kept.source_spans.contains(&span) {
                            kept.source_spans.push(span);
                        }
                    }
                    for id in entity.source_chunk_ids {
                        if !kept.source_chunk_ids.contains(&id) {
                            kept.source_chunk_ids.push(id);
                        }
                    }
                }
                None => {
                    merged.insert(entity.name.clone(), entity);
                }
            }
        }
        result.entities = merged.into_values().collect();

        let before = result.relationships.len();
        for rel in &mut result.relationships {
            if let Some((key, _)) = self.resolve_name(&rel.source) {
                rel.source = key.to_string();
            }
            if let Some((key, _)) = self.resolve_name(&rel.target) {
                rel.target = key.to_string();
            }
        }
        result.relationships.retain(|r| r.source != r.target);
        stats.self_loops_dropped = before - result.relationships.len();
        stats
    }
}

fn system_section(o: &WorkspaceOntology) -> String {
    if o.entity_types.is_empty() && o.relation_types.is_empty() && o.terms.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n## Domain Ontology");
    if !o.name.trim().is_empty() {
        out.push_str(&format!(": {}", o.name.trim()));
    }
    out.push('\n');
    if !o.description.trim().is_empty() {
        out.push_str(&format!("{}\n", short(&o.description)));
    }
    if !o.entity_types.is_empty() {
        out.push_str("Entity type definitions (choose `entity_type` by these meanings):\n");
        for t in &o.entity_types {
            let mut line = format!("- {}", t.id);
            if let Some(p) = &t.parent {
                line.push_str(&format!(" (a kind of {p})"));
            }
            if !t.description.is_empty() {
                line.push_str(&format!(": {}", short(&t.description)));
            }
            if !t.synonyms.is_empty() {
                line.push_str(&format!(" Also called: {}.", t.synonyms.join(", ")));
            }
            line.push('\n');
            if out.len() + line.len() > SYSTEM_SECTION_CHARS {
                break;
            }
            out.push_str(&line);
        }
    }
    if !o.relation_types.is_empty() {
        out.push_str("Relation type definitions:\n");
        for r in &o.relation_types {
            let mut line = format!("- {}", r.id);
            if !r.source_types.is_empty() && !r.target_types.is_empty() {
                line.push_str(&format!(
                    " ({} → {})",
                    r.source_types.join("|"),
                    r.target_types.join("|")
                ));
            }
            if !r.description.is_empty() {
                line.push_str(&format!(": {}", short(&r.description)));
            }
            line.push('\n');
            if out.len() + line.len() > SYSTEM_SECTION_CHARS {
                break;
            }
            out.push_str(&line);
        }
    }
    if !o.terms.is_empty() {
        out.push_str(
            "The workspace has a list of reference terms. When a chunk mentions one, it is listed with the text: use its canonical name and type.\n",
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{ExtractedEntity, ExtractedRelationship};
    use crate::ontology::model::{OntologyEntityType, OntologyTerm};

    fn ontology() -> CompiledOntology {
        let o = validate(
            WorkspaceOntology {
                name: "Propulsion".into(),
                entity_types: vec![
                    OntologyEntityType {
                        id: "ENGINE".into(),
                        description: "A gas turbine engine.".into(),
                        parent: Some("PRODUCT".into()),
                        synonyms: vec!["TURBOFAN".into()],
                        ..Default::default()
                    },
                    OntologyEntityType {
                        id: "PRODUCT".into(),
                        ..Default::default()
                    },
                ],
                terms: vec![OntologyTerm {
                    name: "LEAP-1A".into(),
                    entity_type: Some("ENGINE".into()),
                    aliases: vec!["LEAP 1A".into(), "Leap1A".into()],
                    description: String::new(),
                }],
                ..Default::default()
            },
            ValidationMode::Strict,
        )
        .unwrap()
        .ontology;
        CompiledOntology::compile(&o)
    }

    #[test]
    fn aliases_become_one_canonical_entity_with_the_term_type() {
        let c = ontology();
        let mut r = ExtractionResult::new("chunk-1");
        r.entities
            .push(ExtractedEntity::new("LEAP_1A", "PRODUCT", "First mention."));
        r.entities.push(ExtractedEntity::new(
            "LEAP1A",
            "TURBOFAN",
            "Second mention.",
        ));
        r.entities
            .push(ExtractedEntity::new("AIRBUS", "TURBOFAN", "Not a term."));
        r.relationships
            .push(ExtractedRelationship::new("LEAP_1A", "LEAP1A", "SAME_AS"));
        r.relationships
            .push(ExtractedRelationship::new("LEAP1A", "AIRBUS", "POWERS"));
        let stats = c.canonicalize(&mut r);
        assert_eq!(r.entities.len(), 2);
        assert_eq!(r.entities[0].name, "LEAP-1A");
        assert_eq!(r.entities[0].entity_type, "ENGINE");
        assert_eq!(r.entities[0].description, "First mention.\nSecond mention.");
        // Synonym type mapped even for a name that is not a term.
        assert_eq!(r.entities[1].entity_type, "ENGINE");
        assert_eq!(r.relationships.len(), 1);
        assert_eq!(r.relationships[0].source, "LEAP-1A");
        assert_eq!(stats.merged, 1);
        assert_eq!(stats.self_loops_dropped, 1);
    }

    #[test]
    fn hints_list_only_terms_present_as_whole_words() {
        let c = ontology();
        let hints = c.term_hints_section("The leap 1a powers the A320neo.");
        assert!(hints.contains("\"LEAP-1A\" (ENGINE)"), "{hints}");
        assert!(c.term_hints_section("SLEAP 1AB is not it.").is_empty());
        assert!(c.term_hints_section("Nothing here.").is_empty());
    }

    #[test]
    fn system_section_has_definitions_hierarchy_and_synonyms() {
        let s = ontology().system_prompt_section().to_string();
        assert!(s.contains("## Domain Ontology: Propulsion"));
        assert!(s.contains(
            "- ENGINE (a kind of PRODUCT): A gas turbine engine. Also called: TURBOFAN."
        ));
    }

    #[test]
    fn stored_ontology_never_blocks_extraction() {
        let mut md = HashMap::new();
        assert!(CompiledOntology::from_workspace_metadata(&md).is_none());
        md.insert(METADATA_KEY.to_string(), serde_json::json!("not an object"));
        assert!(CompiledOntology::from_workspace_metadata(&md).is_none());
        md.insert(
            METADATA_KEY.to_string(),
            serde_json::json!({"terms": [{"name": "LEAP-1A"}]}),
        );
        assert!(CompiledOntology::from_workspace_metadata(&md).is_some());
    }
}
