//! SPEC-119 — singular-only edge discovery under 2s discovery wall budget.
//!
//! Run:
//! ```bash
//! export DATABASE_URL=...
//! cargo test -p edgequake-storage --features postgres --test e2e_spec119_singular_edge_discovery -- --nocapture
//! ```

#![cfg(feature = "postgres")]

#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{
    EdgeListFilter, GraphScanOps, GraphStorage, GraphStorageMutateOps,
};
use edgequake_storage::PostgresAGEGraphStorage;
use perf_harness::{assert_plan_uses_index, PlanKind};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Soft functional seed (correctness + modest load).
const SEED_EDGES: usize = 200;
/// Wall must stay under discovery statement timeout.
const WALL_BUDGET_MS: u128 = 2000;
/// EXPLAIN ANALYZE budget for OR singular probe after indexes exist.
const EXPLAIN_ANALYZE_BUDGET_MS: f64 = 500.0;

#[tokio::test]
async fn e2e_spec119_singular_edge_discovery_wall() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("spec119_singular_wall")
    else {
        return;
    };

    std::env::remove_var("EDGEQUAKE_SOURCE_PREFIX_LEGACY");

    let storage = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    storage.initialize().await.expect("graph init");

    let doc = format!("spec119-wall-{}", Uuid::new_v4().simple());
    let chunk = format!("{doc}-chunk-0");
    let tenant = "t-spec119-wall";
    let workspace = "ws-spec119-wall";

    let mut seeded = 0usize;
    for i in 0..SEED_EDGES {
        let a = format!("SPEC119_W_A_{i}_{}", Uuid::new_v4().simple());
        let b = format!("SPEC119_W_B_{i}_{}", Uuid::new_v4().simple());
        let mut np = HashMap::new();
        np.insert("entity_type".into(), json!("concept"));
        np.insert("tenant_id".into(), json!(tenant));
        np.insert("workspace_id".into(), json!(workspace));
        storage.upsert_node(&a, np.clone()).await.expect("node a");
        storage.upsert_node(&b, np).await.expect("node b");

        let mut ep = HashMap::new();
        ep.insert("tenant_id".into(), json!(tenant));
        ep.insert("workspace_id".into(), json!(workspace));
        ep.insert("relation_type".into(), json!("RELATED"));
        // Singular-only — modern GIN path cannot find these.
        ep.insert("source_chunk_id".into(), json!(&chunk));
        ep.insert("source_document_id".into(), json!(&doc));
        storage.upsert_edge(&a, &b, ep).await.expect("edge");
        seeded += 1;
    }

    let filter = EdgeListFilter {
        tenant_id: Some(tenant.into()),
        workspace_id: Some(workspace.into()),
        relationship_type: None,
    };
    let prefixes = vec![doc.clone()];

    let start = Instant::now();
    let hits = storage
        .find_edges_by_source_prefixes(&filter, &prefixes)
        .await
        .expect("singular discovery");
    let elapsed = start.elapsed();

    assert!(
        hits.len() >= seeded,
        "expected ≥{seeded} singular-citation edges, got {}",
        hits.len()
    );
    assert!(
        elapsed < Duration::from_millis(WALL_BUDGET_MS as u64),
        "singular find_edges_by_source_prefixes wall {:?} exceeds {}ms budget (LAW-119-4)",
        elapsed,
        WALL_BUDGET_MS
    );

    // EXPLAIN ANALYZE OR probe — must use indexes and finish well under discovery budget.
    let pool = postgres_test_config::contract_pg_pool(&config).await;
    let graph = storage.graph_name().to_string();
    let explain_sql = format!(
        r#"EXPLAIN (ANALYZE, FORMAT TEXT)
           SELECT 1
           FROM {graph}."EDGE" e
           WHERE ag_catalog.agtype_to_json(e.properties)->>'source_chunk_id' = $1
              OR ag_catalog.agtype_to_json(e.properties)->>'source_document_id' = $2
           LIMIT 5000"#
    );
    let plan_rows: Vec<(String,)> = sqlx::query_as(&explain_sql)
        .bind(&chunk)
        .bind(&doc)
        .fetch_all(&pool)
        .await
        .expect("EXPLAIN ANALYZE OR");
    let plan = plan_rows
        .into_iter()
        .map(|r| r.0)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !plan.contains("_ag_label_edge"),
        "must target child EDGE:\n{plan}"
    );
    assert_plan_uses_index(&plan, &[PlanKind::Btree, PlanKind::Bitmap, PlanKind::Index]);
    // Parse "Execution Time: X ms" when present.
    if let Some(ms) = plan
        .lines()
        .find_map(|l| l.trim().strip_prefix("Execution Time:"))
        .and_then(|rest| {
            rest.trim()
                .strip_suffix("ms")
                .and_then(|n| n.trim().parse::<f64>().ok())
        })
    {
        assert!(
            ms < EXPLAIN_ANALYZE_BUDGET_MS,
            "OR singular EXPLAIN ANALYZE {ms}ms exceeds {EXPLAIN_ANALYZE_BUDGET_MS}ms; plan:\n{plan}"
        );
    }

    eprintln!(
        "OK SPEC-119: singular discovery {:.0}ms (hits={}, seeded={})\nOR EXPLAIN ANALYZE:\n{plan}",
        elapsed.as_secs_f64() * 1000.0,
        hits.len(),
        seeded
    );
}

/// Scale proof: prefer configured graph; fall back to `eq_eq_default_graph` when large.
#[tokio::test]
async fn e2e_spec119_live_graph_singular_index_cond_if_present() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("spec119_live_scale") else {
        return;
    };
    let storage = PostgresAGEGraphStorage::new(config.clone());
    storage.initialize().await.expect("graph init");
    let pool = postgres_test_config::contract_pg_pool(&config).await;

    let mut candidates = vec![storage.graph_name().to_string()];
    if !candidates.iter().any(|g| g == "eq_eq_default_graph") {
        candidates.push("eq_eq_default_graph".into());
    }

    for graph in candidates {
        let n_edges: i64 =
            sqlx::query_scalar(&format!(r#"SELECT COUNT(*)::bigint FROM {graph}."EDGE""#))
                .fetch_one(&pool)
                .await
                .unwrap_or(0);
        if n_edges < 10_000 {
            eprintln!("skip candidate {graph}: {n_edges} edges");
            continue;
        }

        let idx: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*)::bigint FROM pg_indexes
               WHERE schemaname = $1 AND indexname = 'idx_edge_source_chunk_id'"#,
        )
        .bind(&graph)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
        if idx == 0 {
            eprintln!("skip candidate {graph}: missing idx_edge_source_chunk_id");
            continue;
        }

        let probe_sql = format!(
            r#"SELECT ag_catalog.agtype_to_json(properties)->>'source_chunk_id'
               FROM {graph}."EDGE"
               WHERE ag_catalog.agtype_to_json(properties)->>'source_chunk_id' IS NOT NULL
               LIMIT 1"#
        );
        let probe: Option<String> = sqlx::query_scalar(&probe_sql)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();
        let Some(chunk) = probe else {
            eprintln!("skip candidate {graph}: no singular samples");
            continue;
        };

        let plan_sql = format!(
            r#"EXPLAIN (ANALYZE, FORMAT TEXT)
               SELECT 1 FROM {graph}."EDGE" e
               WHERE ag_catalog.agtype_to_json(e.properties)->>'source_chunk_id' = $1
               LIMIT 5000"#
        );
        let plan_rows: Vec<(String,)> = sqlx::query_as(&plan_sql)
            .bind(&chunk)
            .fetch_all(&pool)
            .await
            .expect("live EXPLAIN");
        let plan = plan_rows
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>()
            .join("\n");
        assert_plan_uses_index(&plan, &[PlanKind::Btree, PlanKind::Bitmap, PlanKind::Index]);
        assert!(
            plan.contains("idx_edge_source_chunk_id"),
            "live {n_edges}-edge graph {graph} must use idx_edge_source_chunk_id:\n{plan}"
        );
        eprintln!("OK SPEC-119 live scale ({graph}, {n_edges} edges):\n{plan}");
        return;
    }

    eprintln!("skip live scale: no candidate graph with ≥10000 edges + singular index");
}
