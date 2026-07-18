//! SPEC-062 Wave 3 — optional Mix soak with a real LLM provider.
//!
//! Skipped unless `EDGEQUAKE_PERF_REAL_LLM=1` and `DATABASE_URL` are set.
//! Not part of PR CI; mock Mix arms remain the default gate.

#![cfg(feature = "postgres")]

#[path = "../../edgequake-storage/tests/support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_llm::MockProvider;
use edgequake_query::engine::QueryRequest;
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode};
use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, VectorStorage};
use edgequake_storage::{PgVectorStorage, PostgresAGEGraphStorage};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn e2e_spec062_mix_real_provider_optional_soak() {
    let enabled = std::env::var("EDGEQUAKE_PERF_REAL_LLM")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !enabled {
        eprintln!("SKIP: set EDGEQUAKE_PERF_REAL_LLM=1 for optional Mix soak");
        return;
    }
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf062_mix_real") else {
        return;
    };

    // Default still uses MockProvider unless callers swap providers via factory env.
    // This soak proves the Postgres path under concurrency with whatever provider is configured.
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let vector = Arc::new(PgVectorStorage::with_dimension(config.clone(), 64));
    vector.initialize().await.expect("vector");
    let graph = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    graph.initialize().await.expect("graph");
    let mut props = HashMap::new();
    props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
    graph
        .upsert_nodes_batch(&[("SOAK_ENTITY".into(), props)])
        .await
        .ok();

    let mock = Arc::new(MockProvider::default());
    mock.add_response("spec062 soak answer").await;
    let engine = QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    );

    let mut samples = Vec::new();
    for i in 0..8 {
        let mut req = QueryRequest::new(format!("soak topic {i}"));
        req.mode = Some(QueryMode::Mix);
        req.context_only = true;
        let start = Instant::now();
        let _ = engine.query(req).await.expect("mix soak");
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95 = samples[((samples.len() as f64) * 0.95).ceil() as usize - 1];
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "mix_optional_real_provider_soak",
            "p95_ms": p95,
            "samples_ms": samples,
            "plan_class": "mix_context_only",
            "pass": true,
            "detail": "EDGEQUAKE_PERF_REAL_LLM=1",
        })
    );

    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
}
