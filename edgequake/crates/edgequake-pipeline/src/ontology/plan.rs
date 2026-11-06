//! What realigning an existing graph to an ontology would change (SPEC-184 LAW-184-5).
//!
//! Pure: the caller looks up the graph nodes whose ids are term keys
//! ([`CompiledOntology::graph_keys`]) and passes what it found.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use super::compiled::CompiledOntology;

/// One graph node the plan needs to know about.
#[derive(Debug, Clone)]
pub struct NodeFact {
    pub entity_type: String,
    pub degree: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedMerge {
    /// Existing entity named with an alias.
    pub source: String,
    /// Canonical entity it folds into.
    pub target: String,
    /// `false`: the canonical entity is created first (its type: the term's, else the alias's).
    pub target_exists: bool,
    pub target_type: String,
    pub source_degree: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedRetype {
    pub entity: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AlignmentPlan {
    pub merges: Vec<PlannedMerge>,
    pub retypes: Vec<PlannedRetype>,
    pub terms_total: usize,
    /// Terms present in the graph under their canonical name or an alias.
    pub terms_in_graph: usize,
}

impl AlignmentPlan {
    pub fn is_empty(&self) -> bool {
        self.merges.is_empty() && self.retypes.is_empty()
    }
}

pub fn alignment_plan(
    ontology: &CompiledOntology,
    found: &HashMap<String, NodeFact>,
) -> AlignmentPlan {
    // term index → (canonical present?, aliases present)
    let mut per_term: BTreeMap<usize, (bool, Vec<&str>)> = BTreeMap::new();
    for (key, term) in ontology.graph_keys() {
        if !found.contains_key(key) {
            continue;
        }
        let (canonical, _, _) = ontology.term(term);
        let entry = per_term.entry(term).or_default();
        if key == canonical {
            entry.0 = true;
        } else {
            entry.1.push(key);
        }
    }

    let mut plan = AlignmentPlan {
        terms_total: ontology.term_count(),
        terms_in_graph: per_term.len(),
        ..Default::default()
    };
    for (term, (canonical_exists, mut aliases)) in per_term {
        let (canonical, _, term_type) = ontology.term(term);
        // Largest alias first: it becomes the canonical node when none exists.
        aliases.sort_by(|a, b| found[*b].degree.cmp(&found[*a].degree).then(a.cmp(b)));
        let target_type = term_type
            .map(str::to_string)
            .or_else(|| found.get(canonical).map(|n| n.entity_type.clone()))
            .or_else(|| aliases.first().map(|a| found[*a].entity_type.clone()))
            .unwrap_or_default();
        for alias in &aliases {
            plan.merges.push(PlannedMerge {
                source: alias.to_string(),
                target: canonical.to_string(),
                target_exists: canonical_exists,
                target_type: target_type.clone(),
                source_degree: found[*alias].degree,
            });
        }
        if let (true, Some(to)) = (canonical_exists, term_type) {
            let from = &found[canonical].entity_type;
            if from != to {
                plan.retypes.push(PlannedRetype {
                    entity: canonical.to_string(),
                    from: from.clone(),
                    to: to.to_string(),
                });
            }
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::model::{validate, OntologyTerm, ValidationMode, WorkspaceOntology};

    fn compiled() -> CompiledOntology {
        let o = validate(
            WorkspaceOntology {
                terms: vec![
                    OntologyTerm {
                        name: "LEAP-1A".into(),
                        entity_type: Some("ENGINE".into()),
                        aliases: vec!["LEAP 1A".into(), "Leap1A".into()],
                        ..Default::default()
                    },
                    OntologyTerm {
                        name: "CFM56".into(),
                        entity_type: Some("ENGINE".into()),
                        ..Default::default()
                    },
                    OntologyTerm {
                        name: "Absent".into(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ValidationMode::Strict,
        )
        .unwrap()
        .ontology;
        CompiledOntology::compile(&o)
    }

    fn node(t: &str, d: usize) -> NodeFact {
        NodeFact {
            entity_type: t.into(),
            degree: d,
        }
    }

    #[test]
    fn merges_aliases_and_retypes_existing_canonical_terms() {
        let found = HashMap::from([
            ("LEAP_1A".to_string(), node("PRODUCT", 3)),
            ("LEAP1A".to_string(), node("PRODUCT", 9)),
            ("CFM56".to_string(), node("ARTIFACT", 1)),
        ]);
        let plan = alignment_plan(&compiled(), &found);
        assert_eq!(plan.terms_total, 3);
        assert_eq!(plan.terms_in_graph, 2);
        assert_eq!(plan.merges.len(), 2);
        assert_eq!(plan.merges[0].source, "LEAP1A", "largest alias first");
        assert!(plan
            .merges
            .iter()
            .all(|m| m.target == "LEAP-1A" && !m.target_exists && m.target_type == "ENGINE"));
        assert_eq!(
            plan.retypes,
            vec![PlannedRetype {
                entity: "CFM56".into(),
                from: "ARTIFACT".into(),
                to: "ENGINE".into()
            }]
        );
    }

    #[test]
    fn nothing_to_do_when_the_graph_already_matches() {
        let found = HashMap::from([("LEAP-1A".to_string(), node("ENGINE", 4))]);
        assert!(alignment_plan(&compiled(), &found).is_empty());
    }
}
