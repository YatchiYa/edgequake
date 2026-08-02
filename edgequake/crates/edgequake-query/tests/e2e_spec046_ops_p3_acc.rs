//! SPEC-046 OPS-P3 ACC + optional Mistral live faithfulness judge.
//!
//! Deterministic ACC always runs. Live Mistral small+embed is skipped unless
//! `MISTRAL_API_KEY` is set (and the live test is not ignored when run with
//! `--ignored`).

use edgequake_llm::traits::{EmbeddingProvider, LLMProvider};
use edgequake_llm::MistralProvider;
use edgequake_query::context::{QueryContext, RetrievedChunk};
use edgequake_query::eval::{
    parse_faithfulness_judge_enabled, parse_judge_score, run_spec046_acc_report,
    score_faithfulness_heuristic, score_faithfulness_llm,
};
use edgequake_query::graph_ppr::{parse_graph_walk_mode, GraphWalkMode};

#[test]
fn e2e_ops_p3_acc_full_pass() {
    let report = run_spec046_acc_report();
    assert!(
        report.is_full_pass(),
        "ACC failures: {:?}",
        report
            .checks
            .iter()
            .filter(|c| !c.passed)
            .collect::<Vec<_>>()
    );
    let json = serde_json::to_string_pretty(&report).expect("serialize ACC report");
    assert!(json.contains("graph_walk_default_bfs"));
    assert!(json.contains("faithfulness_heuristic_floor"));
}

#[test]
fn e2e_ops_p3_bfs_default_and_ppr_escape() {
    assert_eq!(parse_graph_walk_mode(""), GraphWalkMode::Bfs);
    assert_eq!(parse_graph_walk_mode("ppr"), GraphWalkMode::Ppr);
    assert_eq!(GraphWalkMode::default(), GraphWalkMode::Bfs);
}

#[test]
fn e2e_ops_p3_judge_parsers_edge_cases() {
    assert!(!parse_faithfulness_judge_enabled(""));
    assert!(parse_faithfulness_judge_enabled("llm"));
    assert_eq!(parse_judge_score("0.42"), Some(0.42));
    assert_eq!(parse_judge_score("1.5"), Some(1.0));
    assert_eq!(parse_judge_score("90"), Some(0.9));
}

fn mistral_key_present() -> bool {
    std::env::var("MISTRAL_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false)
}

/// Live: mistral-small-latest judges a grounded answer; mistral-embed embeds a query.
#[tokio::test]
#[ignore = "requires MISTRAL_API_KEY — live Mistral small + embed"]
async fn e2e_ops_p3_mistral_small_embed_faithfulness_live() {
    if !mistral_key_present() {
        eprintln!("SKIP: MISTRAL_API_KEY not set");
        return;
    }

    let provider = MistralProvider::new(
        std::env::var("MISTRAL_API_KEY").expect("key"),
        "mistral-small-latest".to_string(),
        "mistral-embed".to_string(),
        None,
    )
    .expect("MistralProvider::new");

    // Embedding arm
    let emb = EmbeddingProvider::embed_one(&provider, "Apache AGE graph extension")
        .await
        .expect("mistral-embed");
    assert_eq!(emb.len(), 1024, "mistral-embed must be 1024-dim");
    assert!(emb.iter().any(|v| *v != 0.0), "embedding must be non-zero");

    // Faithfulness judge arm
    let mut ctx = QueryContext::new();
    ctx.add_chunk(RetrievedChunk::new(
        "c1",
        "Apache AGE is a PostgreSQL extension that adds graph database functionality.",
        1.0,
    ));
    let answer = "Apache AGE adds graph database functionality to PostgreSQL.";
    let heuristic = score_faithfulness_heuristic(answer, &ctx);
    assert!(heuristic >= 0.5, "heuristic floor failed: {heuristic}");

    let judged = score_faithfulness_llm(&provider as &dyn LLMProvider, answer, &ctx)
        .await
        .expect("LLM judge should return a parseable score");
    assert!(
        (0.0..=1.0).contains(&judged),
        "judge score out of range: {judged}"
    );
    // Grounded answer should not be scored as fully unfaithful.
    assert!(
        judged >= 0.3,
        "expected grounded answer score ≥ 0.3, got {judged}"
    );
}
