//! SPEC-046 / GraphRAG-Bench-aligned intent routing contract.
//!
//! Evidence (ICLR 2026 GraphRAG-Bench): graphs hurt L1 facts; help L2/L3.
//! Adaptive routing must send factual → Naive and exploratory → Global.

use edgequake_query::keywords::QueryIntent;
use edgequake_query::modes::QueryMode;

#[test]
fn contract_factual_l1_routes_to_naive() {
    assert_eq!(
        QueryIntent::Factual.recommended_mode(),
        QueryMode::Naive,
        "L1 facts must avoid graph tax"
    );
    let intent = QueryIntent::classify_heuristic("What is machine learning?");
    assert_eq!(intent, QueryIntent::Factual);
    assert_eq!(intent.recommended_mode(), QueryMode::Naive);
}

#[test]
fn contract_exploratory_l3_routes_to_global() {
    assert_eq!(
        QueryIntent::Exploratory.recommended_mode(),
        QueryMode::Global,
        "thematic / overview queries need global/relation arm"
    );
}

#[test]
fn contract_relational_routes_to_hybrid() {
    assert_eq!(
        QueryIntent::Relational.recommended_mode(),
        QueryMode::Hybrid
    );
}

#[test]
fn contract_comparative_and_procedural_use_mix() {
    assert_eq!(QueryIntent::Comparative.recommended_mode(), QueryMode::Mix);
    assert_eq!(QueryIntent::Procedural.recommended_mode(), QueryMode::Mix);
}

#[test]
fn contract_hybrid_naming_includes_naive_in_eq() {
    // Documentation contract: Hybrid uses vector search (implies naive arm path)
    assert!(QueryMode::Hybrid.uses_vector_search());
    assert!(QueryMode::Hybrid.uses_graph());
}
