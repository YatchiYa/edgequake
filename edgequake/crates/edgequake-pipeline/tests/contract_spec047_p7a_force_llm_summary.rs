//! SPEC-047 P7a contract: LightRAG FORCE_LLM_SUMMARY_ON_MERGE fragment gate.
//!
//! Pure policy SSOT — no graph I/O. Proves join-below-threshold and LLM-at-N.

use edgequake_pipeline::{
    collect_unique_fragments, decide_description_merge, join_description_fragments,
    split_description_fragments, DescriptionMergeDecision, DescriptionMergePolicy,
    DEFAULT_FORCE_LLM_SUMMARY_ON_MERGE, GRAPH_FIELD_SEP,
};

fn policy(force: usize) -> DescriptionMergePolicy {
    DescriptionMergePolicy::from_parts(true, force, 1200, 0.85, 4096)
}

#[test]
fn contract_default_force_is_lightrag_eight() {
    assert_eq!(DEFAULT_FORCE_LLM_SUMMARY_ON_MERGE, 8);
    assert_eq!(GRAPH_FIELD_SEP, "<SEP>");
}

#[test]
fn contract_below_threshold_joins_without_llm() {
    let existing = (0..6)
        .map(|i| format!("distinct fact number {i} about topic"))
        .collect::<Vec<_>>()
        .join(GRAPH_FIELD_SEP);
    let d = decide_description_merge(&existing, "distinct fact number 6 about topic", &policy(8));
    match d {
        DescriptionMergeDecision::Resolved(s) => {
            let frags = split_description_fragments(&s);
            assert_eq!(frags.len(), 7);
            assert!(!matches!(
                decide_description_merge(&s, "", &policy(8)),
                DescriptionMergeDecision::NeedsLlm { .. }
            ));
        }
        DescriptionMergeDecision::NeedsLlm { fragments } => {
            panic!(
                "expected join for 7 fragments, got NeedsLlm({})",
                fragments.len()
            );
        }
    }
}

#[test]
fn contract_at_threshold_needs_llm() {
    let existing = (0..7)
        .map(|i| format!("unique observation {i} with payload"))
        .collect::<Vec<_>>()
        .join(GRAPH_FIELD_SEP);
    let d = decide_description_merge(&existing, "unique observation 7 with payload", &policy(8));
    match d {
        DescriptionMergeDecision::NeedsLlm { fragments } => assert_eq!(fragments.len(), 8),
        DescriptionMergeDecision::Resolved(s) => panic!("expected NeedsLlm, got {s}"),
    }
}

#[test]
fn contract_pairwise_soft_resume_two_frags_no_llm() {
    // The pre-P7a bug: Jaccard < 0.85 on two distinct strings → LLM every update.
    let d = decide_description_merge(
        "Company X reported revenue growth in Q3",
        "Company X expanded into the APAC market",
        &policy(8),
    );
    assert!(
        matches!(d, DescriptionMergeDecision::Resolved(_)),
        "2 distinct fragments must join without LLM under LightRAG gate"
    );
}

#[test]
fn contract_collect_dedupes_case_insensitive() {
    let frags = collect_unique_fragments("Alpha fact", "alpha fact");
    assert_eq!(frags.len(), 1);
}

#[test]
fn contract_join_split_roundtrip() {
    let joined = join_description_fragments(&["one".into(), "two".into()], 4096);
    assert_eq!(split_description_fragments(&joined), vec!["one", "two"]);
}
