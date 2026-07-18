//! SPEC-060 — Mix arm wall timers (ex-LLM) + typed vector_type contract.
//!
//! Documents Local/Global/Naive arm p95 on a seeded memory corpus and asserts
//! SPEC-058 typed SQL filter remains present in mode sources.
//!
//! ```bash
//! cargo test -p edgequake-query --test e2e_spec060_query_arm_wall_perf -- --nocapture
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use edgequake_llm::MockProvider;
use edgequake_query::engine::QueryRequest;
use edgequake_query::mix_weights::MixWeightOverride;
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode};
use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps, VectorStorage};
use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage};

const DIM: usize = 1536;
const SAMPLES: usize = 10;

fn percentile_p95(sorted: &[u64]) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    sorted[idx.min(sorted.len() - 1)]
}

fn unit_emb(seed: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; DIM];
    let idx = ((seed.abs() as usize) % DIM).max(1);
    v[0] = 0.1;
    v[idx] = 1.0;
    v
}

#[tokio::test]
async fn e2e_spec060_mix_arm_walls_documented() {
    let vector = Arc::new(MemoryVectorStorage::new("spec060_arms", DIM));
    let graph = Arc::new(MemoryGraphStorage::new("spec060_arms"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();

    // Mixed corpus: chunks + entities + relationships
    let mut batch = Vec::new();
    for i in 0..200 {
        batch.push((
            format!("chunk-{i}"),
            unit_emb(i as f32),
            serde_json::json!({
                "type": "chunk",
                "content": format!("chunk body about topic {i}"),
                "document_id": format!("doc-{}", i / 10),
                "workspace_id": "ws-arm060",
            }),
        ));
    }
    for i in 0..50 {
        batch.push((
            format!("ent-{i}"),
            unit_emb(1000.0 + i as f32),
            serde_json::json!({
                "type": "entity",
                "entity_name": format!("ENTITY_{i}"),
                "workspace_id": "ws-arm060",
            }),
        ));
        let mut props = HashMap::new();
        props.insert("entity_type".to_string(), serde_json::json!("CONCEPT"));
        props.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("chunk-{}", i)]),
        );
        props.insert("workspace_id".to_string(), serde_json::json!("ws-arm060"));
        graph
            .upsert_node(&format!("ENTITY_{i}"), props)
            .await
            .unwrap();
    }
    for i in 0..30 {
        batch.push((
            format!("rel-{i}"),
            unit_emb(2000.0 + i as f32),
            serde_json::json!({
                "type": "relationship",
                "src_id": format!("ENTITY_{}", i),
                "tgt_id": format!("ENTITY_{}", (i + 1) % 50),
                "workspace_id": "ws-arm060",
            }),
        ));
    }
    vector.upsert(&batch).await.unwrap();

    let mock = Arc::new(MockProvider::default());
    mock.add_response("arm wall answer").await;
    let engine = QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    );

    let mut local_ms = Vec::new();
    let mut global_ms = Vec::new();
    let mut naive_ms = Vec::new();

    // Force all three arms (disable intent gating via explicit weights).
    let all_arms = MixWeightOverride {
        local: Some(1.0),
        global: Some(1.0),
        naive: Some(1.0),
    };

    // Warm
    let mut warm = QueryRequest::new("topic entity relationship");
    warm.mode = Some(QueryMode::Mix);
    warm.context_only = true;
    warm.mix_weights = Some(all_arms.clone());
    let _ = engine.query(warm).await.expect("warm");

    for s in 0..SAMPLES {
        let start = Instant::now();
        let mut req = QueryRequest::new(format!("topic {s} entity relationship"));
        req.mode = Some(QueryMode::Mix);
        req.context_only = true;
        req.mix_weights = Some(all_arms.clone());
        let resp = engine.query(req).await.expect("mix query");
        let _e2e = start.elapsed();

        let l = resp.stats.arm_local_ms.expect("arm_local_ms in Mix stats");
        let g = resp.stats.arm_global_ms.expect("arm_global_ms in Mix stats");
        let n = resp.stats.arm_naive_ms.expect("arm_naive_ms in Mix stats");
        local_ms.push(l);
        global_ms.push(g);
        naive_ms.push(n);
    }

    local_ms.sort_unstable();
    global_ms.sort_unstable();
    naive_ms.sort_unstable();
    let p95_local = percentile_p95(&local_ms);
    let p95_global = percentile_p95(&global_ms);
    let p95_naive = percentile_p95(&naive_ms);

    // Soft documentation budgets for in-memory corpus (not Postgres ANN).
    assert!(
        Duration::from_millis(p95_local) < Duration::from_secs(2),
        "local arm p95 {p95_local}ms unexpectedly high"
    );
    assert!(
        Duration::from_millis(p95_global) < Duration::from_secs(2),
        "global arm p95 {p95_global}ms unexpectedly high"
    );
    assert!(
        Duration::from_millis(p95_naive) < Duration::from_secs(2),
        "naive arm p95 {p95_naive}ms unexpectedly high"
    );

    eprintln!(
        "OK SPEC-060 arm walls (ex-LLM, memory): local_p95={p95_local}ms \
         global_p95={p95_global}ms naive_p95={p95_naive}ms \
         samples local={local_ms:?} global={global_ms:?} naive={naive_ms:?}"
    );
}

#[test]
fn contract_spec060_local_global_typed_vector_type_sql() {
    let local = include_str!("../src/engine_impl/modes/local.rs");
    let global = include_str!("../src/engine_impl/modes/global.rs");
    assert!(
        local.contains("entity") && local.contains("vector_type"),
        "Local arm must push vector_type=entity (SPEC-058)"
    );
    assert!(
        global.contains("relationship") && global.contains("vector_type"),
        "Global arm must push vector_type=relationship (SPEC-058)"
    );
}
