//! SPEC-046 Science P4 — ACC CI artifact + corpus + bipartite dual-node.
//!
//! Deterministic. Live Mistral path is ignored unless `--ignored` + key.

use edgequake_llm::traits::{EmbeddingProvider, LLMProvider};
use edgequake_llm::MistralProvider;
use edgequake_query::eval::{
    mini_corpus_query_context, run_spec046_acc_report, run_spec046_corpus_acc_report,
    write_spec046_acc_report_json, CorpusAccReport,
};
use edgequake_query::graph_ppr::{parse_graph_walk_mode, GraphWalkMode};
use edgequake_query::kg_chunk_pick::pick_chunks_by_bipartite_ppr;
use edgequake_storage::traits::GraphEdge;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn e2e_science_p4_acc_full_pass_and_json_artifact() {
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
    assert!(report.corpus.as_ref().is_some_and(|c| c.is_full_pass()));

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/spec046-acc");
    std::fs::create_dir_all(&dir).expect("mkdir ACC dir");
    let path = dir.join("acc_report.json");
    write_spec046_acc_report_json(&path).expect("write ACC JSON");
    let raw = std::fs::read_to_string(&path).expect("read ACC JSON");
    assert!(raw.contains("bipartite_dual_node"));
    assert!(raw.contains("corpus_retrieval_acc"));
    let parsed: edgequake_query::eval::AccReport =
        serde_json::from_str(&raw).expect("ACC JSON deserializes");
    assert!(parsed.is_full_pass());
}

#[test]
fn e2e_science_p4_corpus_acc_full_pass() {
    let report: CorpusAccReport = run_spec046_corpus_acc_report();
    assert_eq!(report.failed, 0, "failures: {:?}", report.cases);
    assert!(report.total >= 5);
}

#[test]
fn e2e_science_p4_bipartite_pick_from_context() {
    let ctx = mini_corpus_query_context();
    let edges: Vec<GraphEdge> = ctx
        .relationships
        .iter()
        .map(|r| GraphEdge {
            source: r.source.clone(),
            target: r.target.clone(),
            properties: HashMap::new(),
        })
        .collect();
    let ranked = pick_chunks_by_bipartite_ppr(&ctx, &edges, 5);
    assert!(
        ranked
            .iter()
            .any(|c| c == "chunk_normandy" || c == "chunk_hinze_pact"),
        "expected mini-corpus chunks in bipartite pick: {ranked:?}"
    );
<<<<<<< HEAD
    assert_eq!(parse_graph_walk_mode(""), GraphWalkMode::Ppr);
=======
    assert_eq!(parse_graph_walk_mode(""), GraphWalkMode::Bfs);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

fn mistral_key_present() -> bool {
    std::env::var("MISTRAL_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false)
}

/// Live: mistral-embed dimensions + small judge on corpus evidence snippet.
#[tokio::test]
#[ignore = "requires MISTRAL_API_KEY — live Mistral small + embed"]
async fn e2e_science_p4_mistral_corpus_embed_live() {
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

    let emb = EmbeddingProvider::embed_one(&provider, "Mont St. Michel Normandy")
        .await
        .expect("embed");
    assert_eq!(emb.len(), 1024);

    let corpus = run_spec046_corpus_acc_report();
    assert!(corpus.is_full_pass());
    // Smoke: LLM can echo a grounded float score for a corpus gold answer.
    let prompt = format!(
        "Reply with only a float 0.0-1.0. Is this answer grounded?\nANSWER: {}\nEVIDENCE: {}",
        corpus.cases[0].id, "Mont St. Michel stands in Normandy"
    );
    let resp = LLMProvider::complete(&provider, &prompt)
        .await
        .expect("complete");
    assert!(!resp.content.trim().is_empty());
}
