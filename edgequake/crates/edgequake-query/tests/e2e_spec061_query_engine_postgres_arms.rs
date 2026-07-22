//! SPEC-061 — Mix/Hybrid arm walls on live Postgres (ex-LLM).
#![cfg(feature = "postgres")]

#[path = "../../edgequake-storage/tests/support/postgres_test_config.rs"]
mod postgres_test_config;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use edgequake_llm::MockProvider;
use edgequake_query::engine::QueryRequest;
use edgequake_query::mix_weights::MixWeightOverride;
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode};
use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, VectorStorage};
use edgequake_storage::{PgVectorStorage, PostgresAGEGraphStorage};

const SAMPLES: usize = 8;

fn emb(seed: f32) -> Vec<f32> {
    (0..1536)
        .map(|i| ((i as f32 + seed) * 0.011).sin())
        .collect()
}

fn percentile_p95(sorted: &[u64]) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    sorted[idx.min(sorted.len() - 1)]
}

#[tokio::test]
async fn e2e_spec061_postgres_mix_hybrid_arm_walls() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf061_qe") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let vector = Arc::new(PgVectorStorage::with_dimension(config.clone(), 1536));
    vector.initialize().await.expect("vector init");
    let graph = Arc::new(PostgresAGEGraphStorage::new(config));
    graph.initialize().await.expect("graph init");

    let mut batch = Vec::new();
    for i in 0..120 {
        batch.push((
            format!("chunk061-{i}"),
            emb(i as f32),
            serde_json::json!({
                "type": "chunk",
                "content": format!("topic entity relationship chunk {i}"),
                "document_id": format!("doc-{}", i / 10),
                "workspace_id": "ws-061",
            }),
        ));
    }
    for i in 0..40 {
        batch.push((
            format!("ent061-{i}"),
            emb(500.0 + i as f32),
            serde_json::json!({
                "type": "entity",
                "entity_name": format!("ENTITY_{i}"),
                "workspace_id": "ws-061",
            }),
        ));
        let mut props = HashMap::new();
        props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
        props.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("chunk061-{i}")]),
        );
        props.insert("workspace_id".to_string(), serde_json::json!("ws-061"));
        graph
            .upsert_node(&format!("ENTITY_{i}"), props)
            .await
            .expect("node");
    }
    for i in 0..20 {
        batch.push((
            format!("rel061-{i}"),
            emb(900.0 + i as f32),
            serde_json::json!({
                "type": "relationship",
                "src_id": format!("ENTITY_{}", i),
                "tgt_id": format!("ENTITY_{}", (i + 1) % 40),
                "workspace_id": "ws-061",
            }),
        ));
    }
    vector.upsert(&batch).await.expect("upsert vectors");

    let mock = Arc::new(MockProvider::default());
    mock.add_response("spec061 arm answer").await;
    let engine = QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    );

    let all_arms = MixWeightOverride {
        local: Some(1.0),
        global: Some(1.0),
        naive: Some(1.0),
    };

    let mut local_ms = Vec::new();
    let mut global_ms = Vec::new();
    let mut naive_ms = Vec::new();

    // Mix: force all three arms via explicit weights.
    for s in 0..SAMPLES {
        let mut req = QueryRequest::new(format!("topic entity {s}"));
        req.mode = Some(QueryMode::Mix);
        req.context_only = true;
        req.mix_weights = Some(all_arms.clone());
        let resp = engine.query(req).await.expect("mix");
        local_ms.push(resp.stats.arm_local_ms.expect("arm_local_ms"));
        global_ms.push(resp.stats.arm_global_ms.expect("arm_global_ms"));
        naive_ms.push(resp.stats.arm_naive_ms.expect("arm_naive_ms"));
    }
    local_ms.sort_unstable();
    global_ms.sort_unstable();
    naive_ms.sort_unstable();
    let pl = percentile_p95(&local_ms);
    let pg = percentile_p95(&global_ms);
    let pn = percentile_p95(&naive_ms);
    assert!(Duration::from_millis(pl) < Duration::from_secs(5));
    assert!(Duration::from_millis(pg) < Duration::from_secs(5));
    assert!(Duration::from_millis(pn) < Duration::from_secs(5));
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "query_engine_Mix_arms",
            "p95_ms": {"local": pl, "global": pg, "naive": pn},
            "samples_ms": {"local": local_ms, "global": global_ms, "naive": naive_ms},
            "plan_class": "mix_context_only_mock_llm",
            "pass": true,
            "detail": "mock_provider context_only",
        })
    );
    eprintln!("OK SPEC-061 Postgres Mix arms: local_p95={pl}ms global_p95={pg}ms naive_p95={pn}ms");

    // Hybrid: document whatever arms ran (intent mask may skip some).
    let mut hybrid_naive = Vec::new();
    for s in 0..SAMPLES {
        let mut req = QueryRequest::new(format!("topic entity hybrid {s}"));
        req.mode = Some(QueryMode::Hybrid);
        req.context_only = true;
        req.mix_weights = Some(all_arms.clone());
        let resp = engine.query(req).await.expect("hybrid");
        if let Some(n) = resp.stats.arm_naive_ms {
            hybrid_naive.push(n);
        }
        // At least one arm wall must be present
        assert!(
            resp.stats.arm_local_ms.is_some()
                || resp.stats.arm_global_ms.is_some()
                || resp.stats.arm_naive_ms.is_some(),
            "Hybrid must record at least one arm wall"
        );
    }
    hybrid_naive.sort_unstable();
    let hn = percentile_p95(&hybrid_naive);
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "query_engine_Hybrid_arms",
            "p95_ms": {"naive": hn},
            "samples_ms": {"naive": hybrid_naive},
            "plan_class": "hybrid_context_only_mock_llm",
            "pass": true,
            "detail": "mock_provider context_only",
        })
    );
    eprintln!(
        "OK SPEC-061 Postgres Hybrid arms: naive_p95={hn}ms (samples={})",
        hybrid_naive.len()
    );

    // Typed filter contract still present
    let local_src = include_str!("../src/engine_impl/modes/local.rs");
    let global_src = include_str!("../src/engine_impl/modes/global.rs");
    assert!(local_src.contains("vector_type") && local_src.contains("entity"));
    assert!(global_src.contains("vector_type") && global_src.contains("relationship"));

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
}
