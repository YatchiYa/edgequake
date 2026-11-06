//! Workspace ontology document and its validation (SPEC-184).

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::prompts::{normalize_entity_name, normalize_type_token};

/// Workspace metadata key holding the ontology document.
pub const METADATA_KEY: &str = "ontology";
/// Workspace metadata key holding the SPEC-114 lists an ontology replaced.
pub const SCHEMA_BEFORE_KEY: &str = "ontology_schema_before";

pub const MAX_ENTITY_TYPES: usize = 50;
pub const MAX_RELATION_TYPES: usize = 50;
pub const MAX_TERMS: usize = 5_000;
pub const MAX_ALIASES: usize = 20;
pub const MAX_SYNONYMS: usize = 20;
pub const MAX_RELATION_EDGES: usize = 100;
const MAX_NAME_CHARS: usize = 200;
const MAX_DESCRIPTION_CHARS: usize = 1_000;

/// One workspace ontology (stored as JSON in workspace metadata).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOntology {
    #[serde(default = "version_one")]
    pub version: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default)]
    pub entity_types: Vec<OntologyEntityType>,
    #[serde(default)]
    pub relation_types: Vec<OntologyRelationType>,
    #[serde(default)]
    pub terms: Vec<OntologyTerm>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<OntologySource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

fn version_one() -> u32 {
    1
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OntologyEntityType {
    #[serde(default)]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OntologyRelationType {
    #[serde(default)]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_types: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OntologyTerm {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OntologySource {
    /// `file`, `document`, `text`, `copilot` or `manual`.
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

/// How [`validate`] treats problems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Saving: ambiguity and over-limit input are errors (LAW-184-4).
    Strict,
    /// Drafting: drop what conflicts or overflows and say so in warnings.
    Repair,
}

/// A clean ontology plus what was changed to get there.
#[derive(Debug, Clone)]
pub struct Validated {
    pub ontology: WorkspaceOntology,
    pub warnings: Vec<String>,
}

fn clip(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        trimmed.to_string()
    } else {
        trimmed.chars().take(max).collect()
    }
}

/// Normalize, de-duplicate and check an ontology.
///
/// Ids become UPPER_SNAKE (`normalize_type_token`); term keys use
/// `normalize_entity_name`, the graph node id normalizer (LAW-184-3).
pub fn validate(input: WorkspaceOntology, mode: ValidationMode) -> Result<Validated, Vec<String>> {
    let repair = mode == ValidationMode::Repair;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let problem = |msg: String, warnings: &mut Vec<String>, errors: &mut Vec<String>| {
        if repair {
            warnings.push(msg);
        } else {
            errors.push(msg);
        }
    };

    // ---- entity types ----
    let mut types: Vec<OntologyEntityType> = Vec::new();
    for raw in input.entity_types {
        let id = normalize_type_token(if raw.id.trim().is_empty() {
            &raw.label
        } else {
            &raw.id
        });
        if id.is_empty() {
            warnings.push("An entity type without an id was dropped.".into());
            continue;
        }
        let synonyms: Vec<String> = raw
            .synonyms
            .iter()
            .map(|s| normalize_type_token(s))
            .collect();
        if let Some(existing) = types.iter_mut().find(|t| t.id == id) {
            warnings.push(format!(
                "Entity type {id} was listed twice; the entries were merged."
            ));
            existing.synonyms.extend(synonyms);
            if existing.description.is_empty() {
                existing.description = clip(&raw.description, MAX_DESCRIPTION_CHARS);
            }
            continue;
        }
        types.push(OntologyEntityType {
            label: clip(&raw.label, MAX_NAME_CHARS),
            description: clip(&raw.description, MAX_DESCRIPTION_CHARS),
            parent: raw
                .parent
                .as_deref()
                .map(normalize_type_token)
                .filter(|p| !p.is_empty() && *p != id),
            synonyms,
            id,
        });
    }
    if types.len() > MAX_ENTITY_TYPES {
        problem(
            format!(
                "{} entity types; the limit is {MAX_ENTITY_TYPES}.",
                types.len()
            ),
            &mut warnings,
            &mut errors,
        );
        types.truncate(MAX_ENTITY_TYPES);
    }
    let type_ids: HashSet<String> = types.iter().map(|t| t.id.clone()).collect();

    // Synonyms: unique, never another type's id, never claimed by two types.
    let mut synonym_owner: HashMap<String, String> = HashMap::new();
    for t in &mut types {
        let mut kept = Vec::new();
        for s in std::mem::take(&mut t.synonyms) {
            if s.is_empty() || s == t.id || kept.contains(&s) {
                continue;
            }
            if type_ids.contains(&s) {
                problem(
                    format!("Synonym {s} of {} is itself an entity type.", t.id),
                    &mut warnings,
                    &mut errors,
                );
                continue;
            }
            if let Some(owner) = synonym_owner.get(&s) {
                problem(
                    format!("Synonym {s} is claimed by both {owner} and {}.", t.id),
                    &mut warnings,
                    &mut errors,
                );
                continue;
            }
            synonym_owner.insert(s.clone(), t.id.clone());
            kept.push(s);
        }
        if kept.len() > MAX_SYNONYMS {
            warnings.push(format!("{} keeps its first {MAX_SYNONYMS} synonyms.", t.id));
            kept.truncate(MAX_SYNONYMS);
        }
        t.synonyms = kept;
    }
    let resolve_type = |raw: &str| -> Option<String> {
        let n = normalize_type_token(raw);
        if type_ids.contains(&n) {
            Some(n)
        } else {
            synonym_owner.get(&n).cloned()
        }
    };

    // Parents: known, and no cycle.
    for t in &mut types {
        if let Some(parent) = t.parent.clone() {
            match resolve_type(&parent) {
                Some(p) => t.parent = Some(p),
                None => {
                    warnings.push(format!(
                        "Parent {parent} of {} is not an entity type of the ontology; it was removed.",
                        t.id
                    ));
                    t.parent = None;
                }
            }
        }
    }
    let parents: HashMap<String, String> = types
        .iter()
        .filter_map(|t| t.parent.clone().map(|p| (t.id.clone(), p)))
        .collect();
    for t in &mut types {
        let mut seen = HashSet::from([t.id.clone()]);
        let mut cursor = parents.get(&t.id);
        while let Some(p) = cursor {
            if !seen.insert(p.clone()) {
                problem(
                    format!("The parent chain of {} loops back on itself.", t.id),
                    &mut warnings,
                    &mut errors,
                );
                t.parent = None;
                break;
            }
            cursor = parents.get(p);
        }
    }

    // ---- relation types ----
    let mut relations: Vec<OntologyRelationType> = Vec::new();
    for raw in input.relation_types {
        let id = normalize_type_token(&raw.id);
        if id.is_empty() {
            warnings.push("A relation type without an id was dropped.".into());
            continue;
        }
        if relations.iter().any(|r| r.id == id) {
            warnings.push(format!(
                "Relation type {id} was listed twice; the first entry is kept."
            ));
            continue;
        }
        let mut endpoints = |list: &[String], side: &str| -> Vec<String> {
            let mut out = Vec::new();
            for raw_type in list {
                match resolve_type(raw_type) {
                    Some(t) if !out.contains(&t) => out.push(t),
                    Some(_) => {}
                    None => warnings.push(format!(
                        "{side} type {raw_type} of relation {id} is not an entity type of the ontology; it was removed."
                    )),
                }
            }
            out
        };
        let source_types = endpoints(&raw.source_types, "Source");
        let target_types = endpoints(&raw.target_types, "Target");
        relations.push(OntologyRelationType {
            description: clip(&raw.description, MAX_DESCRIPTION_CHARS),
            source_types,
            target_types,
            id,
        });
    }
    if relations.len() > MAX_RELATION_TYPES {
        problem(
            format!(
                "{} relation types; the limit is {MAX_RELATION_TYPES}.",
                relations.len()
            ),
            &mut warnings,
            &mut errors,
        );
        relations.truncate(MAX_RELATION_TYPES);
    }

    // ---- terms ----
    let mut terms: Vec<OntologyTerm> = Vec::new();
    let mut term_by_key: HashMap<String, usize> = HashMap::new();
    for raw in input.terms {
        let name = clip(&raw.name, MAX_NAME_CHARS);
        let key = normalize_entity_name(&name);
        if key.is_empty() {
            if !name.is_empty() {
                warnings.push(format!(
                    "Term \"{name}\" cannot be a graph name and was dropped."
                ));
            }
            continue;
        }
        let entity_type = match raw
            .entity_type
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            None => None,
            Some(t) if type_ids.is_empty() => {
                Some(normalize_type_token(t)).filter(|t| !t.is_empty())
            }
            Some(t) => match resolve_type(t) {
                Some(id) => Some(id),
                None => {
                    warnings.push(format!(
                        "Type {t} of term \"{name}\" is not an entity type of the ontology; the term keeps no type."
                    ));
                    None
                }
            },
        };
        let aliases: Vec<String> = raw
            .aliases
            .iter()
            .map(|a| clip(a, MAX_NAME_CHARS))
            .filter(|a| !a.is_empty())
            .collect();
        if let Some(&i) = term_by_key.get(&key) {
            warnings.push(format!(
                "Term \"{name}\" was listed twice; the entries were merged."
            ));
            let existing = &mut terms[i];
            existing.aliases.extend(aliases);
            if existing.entity_type.is_none() {
                existing.entity_type = entity_type;
            }
            if existing.description.is_empty() {
                existing.description = clip(&raw.description, MAX_DESCRIPTION_CHARS);
            }
            continue;
        }
        term_by_key.insert(key, terms.len());
        terms.push(OntologyTerm {
            name,
            entity_type,
            aliases,
            description: clip(&raw.description, MAX_DESCRIPTION_CHARS),
        });
    }
    if terms.len() > MAX_TERMS {
        problem(
            format!("{} terms; the limit is {MAX_TERMS}.", terms.len()),
            &mut warnings,
            &mut errors,
        );
        terms.truncate(MAX_TERMS);
        term_by_key.retain(|_, i| *i < MAX_TERMS);
    }

    // Aliases: one alias → one term (LAW-184-4).
    let mut alias_owner: HashMap<String, usize> = HashMap::new();
    for i in 0..terms.len() {
        let own_key = normalize_entity_name(&terms[i].name);
        let mut kept: Vec<String> = Vec::new();
        let mut kept_keys: HashSet<String> = HashSet::new();
        for alias in std::mem::take(&mut terms[i].aliases) {
            let key = normalize_entity_name(&alias);
            if key.is_empty() || key == own_key || kept_keys.contains(&key) {
                continue;
            }
            let other = term_by_key
                .get(&key)
                .copied()
                .or_else(|| alias_owner.get(&key).copied());
            if let Some(j) = other.filter(|&j| j != i) {
                problem(
                    format!(
                        "Alias \"{alias}\" of \"{}\" already names \"{}\".",
                        terms[i].name, terms[j].name
                    ),
                    &mut warnings,
                    &mut errors,
                );
                continue;
            }
            alias_owner.insert(key.clone(), i);
            kept_keys.insert(key);
            kept.push(alias);
        }
        if kept.len() > MAX_ALIASES {
            warnings.push(format!(
                "\"{}\" keeps its first {MAX_ALIASES} aliases.",
                terms[i].name
            ));
            kept.truncate(MAX_ALIASES);
        }
        terms[i].aliases = kept;
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Validated {
        ontology: WorkspaceOntology {
            version: 1,
            name: clip(&input.name, MAX_NAME_CHARS),
            description: clip(&input.description, MAX_DESCRIPTION_CHARS * 2),
            entity_types: types,
            relation_types: relations,
            terms,
            sources: input.sources,
            updated_at: input.updated_at,
        },
        warnings,
    })
}

/// SPEC-114 lists derived from an ontology (written on save).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SchemaProjection {
    /// `None` when the ontology lists no entity types (keep the workspace's own).
    pub entity_types: Option<Vec<String>>,
    pub relation_types: Option<Vec<String>>,
    /// `(source, relation, target)`; `None` when no relation names endpoint types.
    pub relation_edges: Option<Vec<(String, String, String)>>,
    /// Pairs dropped beyond [`MAX_RELATION_EDGES`].
    pub dropped_edges: usize,
}

pub fn schema_projection(ontology: &WorkspaceOntology) -> SchemaProjection {
    let entity_types = (!ontology.entity_types.is_empty())
        .then(|| ontology.entity_types.iter().map(|t| t.id.clone()).collect());
    let relation_types = (!ontology.relation_types.is_empty()).then(|| {
        ontology
            .relation_types
            .iter()
            .map(|r| r.id.clone())
            .collect()
    });
    let mut edges = Vec::new();
    let mut dropped = 0;
    for r in &ontology.relation_types {
        for s in &r.source_types {
            for t in &r.target_types {
                if edges.len() < MAX_RELATION_EDGES {
                    edges.push((s.clone(), r.id.clone(), t.clone()));
                } else {
                    dropped += 1;
                }
            }
        }
    }
    SchemaProjection {
        entity_types,
        relation_edges: (relation_types.is_some() && !edges.is_empty()).then_some(edges),
        relation_types,
        dropped_edges: dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(name: &str, ty: Option<&str>, aliases: &[&str]) -> OntologyTerm {
        OntologyTerm {
            name: name.into(),
            entity_type: ty.map(Into::into),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            description: String::new(),
        }
    }

    fn ty(id: &str, parent: Option<&str>, synonyms: &[&str]) -> OntologyEntityType {
        OntologyEntityType {
            id: id.into(),
            parent: parent.map(Into::into),
            synonyms: synonyms.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn normalizes_ids_and_resolves_synonyms_in_terms() {
        let v = validate(
            WorkspaceOntology {
                entity_types: vec![
                    ty("engine", Some("product"), &["turbofan"]),
                    ty("Product", None, &[]),
                ],
                terms: vec![term("LEAP-1A", Some("Turbofan"), &["LEAP 1A", "leap-1a"])],
                ..Default::default()
            },
            ValidationMode::Strict,
        )
        .unwrap();
        let o = v.ontology;
        assert_eq!(o.entity_types[0].id, "ENGINE");
        assert_eq!(o.entity_types[0].parent.as_deref(), Some("PRODUCT"));
        assert_eq!(o.entity_types[0].synonyms, vec!["TURBOFAN"]);
        assert_eq!(o.terms[0].entity_type.as_deref(), Some("ENGINE"));
        // "leap-1a" normalizes to the term's own key: dropped as redundant.
        assert_eq!(o.terms[0].aliases, vec!["LEAP 1A"]);
    }

    #[test]
    fn ambiguity_is_an_error_when_saving_and_dropped_when_drafting() {
        let input = WorkspaceOntology {
            terms: vec![term("Alpha", None, &["A1"]), term("Beta", None, &["a1"])],
            ..Default::default()
        };
        let errors = validate(input.clone(), ValidationMode::Strict).unwrap_err();
        assert!(errors[0].contains("already names"), "{errors:?}");
        let repaired = validate(input, ValidationMode::Repair).unwrap();
        assert!(repaired.ontology.terms[1].aliases.is_empty());
        assert_eq!(repaired.warnings.len(), 1);
    }

    #[test]
    fn parent_cycles_and_type_synonym_collisions_are_errors() {
        let cycle = WorkspaceOntology {
            entity_types: vec![ty("A", Some("B"), &[]), ty("B", Some("A"), &[])],
            ..Default::default()
        };
        assert!(validate(cycle, ValidationMode::Strict).is_err());
        let collision = WorkspaceOntology {
            entity_types: vec![ty("A", None, &["B"]), ty("B", None, &[])],
            ..Default::default()
        };
        assert!(validate(collision, ValidationMode::Strict).is_err());
    }

    #[test]
    fn projection_builds_typed_edges_only_when_endpoints_are_named() {
        let o = validate(
            WorkspaceOntology {
                entity_types: vec![ty("PART", None, &[]), ty("ENGINE", None, &[])],
                relation_types: vec![
                    OntologyRelationType {
                        id: "part_of".into(),
                        source_types: vec!["PART".into()],
                        target_types: vec!["ENGINE".into()],
                        ..Default::default()
                    },
                    OntologyRelationType {
                        id: "MENTIONS".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ValidationMode::Strict,
        )
        .unwrap()
        .ontology;
        let p = schema_projection(&o);
        assert_eq!(p.entity_types.unwrap(), vec!["PART", "ENGINE"]);
        assert_eq!(p.relation_types.unwrap(), vec!["PART_OF", "MENTIONS"]);
        assert_eq!(
            p.relation_edges.unwrap(),
            vec![("PART".into(), "PART_OF".into(), "ENGINE".into())]
        );
        assert_eq!(
            schema_projection(&WorkspaceOntology::default()),
            SchemaProjection::default()
        );
    }
}
