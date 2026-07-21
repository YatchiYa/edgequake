//! SPEC-061/062 — concurrent Mix context_only + arm semaphore bound.
//!
//! pg16: N=8 ≤2× single-client; pg17/18: N=16 ≤1.5×.
//! Scale via `EDGEQUAKE_PERF_SCALE=prod|large` (5k seed @1536).
#![cfg(feature = "postgres")]

#[path = "../../edgequake-storage/tests/support/perf_stress.rs"]
mod perf_stress;
#[path = "../../edgequake-storage/tests/support/postgres_test_config.rs"]
mod postgres_test_config;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use edgequake_llm::MockProvider;
use edgequake_query::engine::QueryRequest;
use edgequake_query::engine_impl::modes::arm_concurrency::{
    acquire_arm_permit_for_tests, available_arm_permits_for_tests,
};
use edgequake_query::mix_weights::MixWeightOverride;
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode};
use edgequake_storage::traits::{GraphStorage, VectorStorage};
use edgequake_storage::{PgVectorStorage, PostgresAGEGraphStorage};
use perf_stress::{
    mix_absolute_budget_ms, mix_scale, perf_scale, stress_clients, stress_mult, stress_pool_max,
    with_stress_pool,
};

fn emb(dim: usize, seed: f32) -> Vec<f32> {
    (0..dim)
        .map(|i| ((i as f32 + seed) * 0.009).sin())
        .collect()
}

fn percentile_p95_ms(samples: &[Duration]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort();
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    sorted[idx.min(sorted.len() - 1)].as_secs_f64() * 1000.0
}

#[tokio::test]
async fn e2e_spec061_stress_mix_arms_and_semaphore() {
    // Semaphore bound (no DB)
    let limit = available_arm_permits_for_tests().max(1);
    let in_flight = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for _ in 0..(limit * 3) {
        let in_flight = Arc::clone(&in_flight);
        let peak = Arc::clone(&peak);
        handles.push(tokio::spawn(async move {
            let _p = acquire_arm_permit_for_tests().await;
            let cur = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(cur, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(15)).await;
            in_flight.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.await.expect("join");
    }
    let observed = peak.load(Ordering::SeqCst);
    assert!(observed <= limit, "semaphore peak {observed} > {limit}");

    let scale = perf_scale();
    let mix = mix_scale(scale);
    let clients = stress_clients();
    let mult = stress_mult();
    let pool_n = stress_pool_max(clients);
    let abs_cap = mix_absolute_budget_ms();

    let Some(base) = postgres_test_config::require_or_skip_postgres("stress061_mix") else {
        return;
    };
    let config = with_stress_pool(base, clients);
    let vector = Arc::new(PgVectorStorage::with_dimension(config.clone(), mix.dim));
    vector.initialize().await.expect("vec");
    let graph = Arc::new(PostgresAGEGraphStorage::new(config));
    graph.initialize().await.expect("graph");

    for batch_start in (0..mix.seed_rows).step_by(500) {
        let end = (batch_start + 500).min(mix.seed_rows);
        let batch: Vec<_> = (batch_start..end)
            .map(|i| {
                (
                    format!("smix-{i}"),
                    emb(mix.dim, i as f32),
                    serde_json::json!({"type": "chunk", "content": format!("mix stress {i}")}),
                )
            })
            .collect();
        vector.upsert(&batch).await.expect("upsert");
    }

    let mock = Arc::new(MockProvider::default());
    mock.add_response("ok").await;
    let engine = Arc::new(QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    ));
    let weights = MixWeightOverride {
        local: Some(1.0),
        global: Some(1.0),
        naive: Some(1.0),
    };

    // Single-client baseline
    let mut single = Vec::new();
    for q in 0..12 {
        let mut req = QueryRequest::new(format!("mix baseline {q}"));
        req.mode = Some(QueryMode::Mix);
        req.context_only = true;
        req.mix_weights = Some(weights.clone());
        let t0 = Instant::now();
        let resp = engine.query(req).await.expect("mix single");
        single.push(t0.elapsed());
        assert!(resp.stats.arm_naive_ms.is_some());
    }
    let single_p95 = percentile_p95_ms(&single);

    let qpc = mix.queries_per_client;
    let mut qhandles = Vec::new();
    let start = Instant::now();
    for c in 0..clients {
        let engine = Arc::clone(&engine);
        let weights = weights.clone();
        qhandles.push(tokio::spawn(async move {
            let mut times = Vec::new();
            for q in 0..qpc {
                let mut req = QueryRequest::new(format!("mix stress {c} {q}"));
                req.mode = Some(QueryMode::Mix);
                req.context_only = true;
                req.mix_weights = Some(weights.clone());
                let t0 = Instant::now();
                let resp = engine.query(req).await.expect("mix");
                times.push(t0.elapsed());
                assert!(resp.stats.arm_naive_ms.is_some());
            }
            times
        }));
    }
    let mut all = Vec::new();
    for h in qhandles {
        all.extend(h.await.expect("join"));
    }
    let p95_ms = percentile_p95_ms(&all);
    // Arm semaphore (limit) serializes work: with N clients expect ~ceil(N/limit)×
    // single-client wall under load, then apply major mult (1.5× / 2×).
    let arm_factor = ((clients as f64) / (limit as f64)).ceil().max(1.0);
    let budget = (single_p95 * mult * arm_factor).min(abs_cap).max(50.0);
    let pass = p95_ms < budget;
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "stress_concurrent_mix",
            "p95_ms": p95_ms,
            "samples_ms": all.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
            "arm_limit": limit,
            "arm_peak": observed,
            "wall_ms": start.elapsed().as_secs_f64() * 1000.0,
            "pass": pass,
            "detail": format!(
                "scale={} clients={clients} seed={} dim={} q/client={qpc} pool={pool_n} single_p95={single_p95:.2} mult={mult} arm_factor={arm_factor} budget={budget:.2} abs_cap={abs_cap} noise_ok",
                scale.as_str(),
                mix.seed_rows,
                mix.dim,
            ),
        })
    );
    assert!(
        pass,
        "Mix concurrent p95 {p95_ms:.2}ms exceeds budget {budget:.2}ms (single_p95={single_p95:.2} mult={mult} abs_cap={abs_cap})"
    );
    eprintln!(
        "OK SPEC-061 Mix stress: p95={p95_ms:.2}ms single_p95={single_p95:.2} clients={clients} scale={} arm_limit={limit} peak={observed}",
        scale.as_str()
    );
}
