//! SPEC-063 — capacity ladder L1/L2/L3 filtered ANN soak (not PR CI).
//!
//! Env:
//! - `EDGEQUAKE_PERF_SCALE=large` (required)
//! - `EDGEQUAKE_CAPACITY_LADDER=L1|L2|L3` (default L1 → 100k @1536)
//! - Prefer `cargo test --release`
//!
//! Honesty: Q1-d SLO (p95&lt;500ms) is reported separately. Ladder **completes**
//! if under hang cliff and stress ≤1.5×/2× single; SLO miss demotes the
//! “supported” claim but still archives a measured cliff.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_stress.rs"]
mod perf_stress;
#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType};
use perf_ann_corpus::{measure_single, measure_stress, seed_ws_split, workspace_filter};
use perf_harness::finish_report;
use perf_stress::{
    ann_scale, capacity_ladder, perf_scale, stress_clients, stress_mult, stress_pool_max,
    with_stress_pool, CapacityLadder, PerfScale,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TOP_K: usize = 20;
const Q1D_SLO_MS: f64 = 500.0;

fn hang_cliff_ms(ladder: CapacityLadder) -> f64 {
    match ladder {
        CapacityLadder::L1 => 5_000.0,
        CapacityLadder::L2 => 10_000.0,
        CapacityLadder::L3 => 20_000.0,
    }
}

#[tokio::test]
async fn e2e_spec063_capacity_ladder_filtered_ann() {
    let scale = perf_scale();
    assert_eq!(
        scale,
        PerfScale::Large,
        "set EDGEQUAKE_PERF_SCALE=large for capacity ladder (got {})",
        scale.as_str()
    );
    let ladder = capacity_ladder();
    let ann = ann_scale(scale);
    let clients = stress_clients();
    let mult = stress_mult();
    let pool = stress_pool_max(clients);
    let cliff = hang_cliff_ms(ladder);

    let Some(base) = postgres_test_config::require_or_skip_postgres("cap063_ann") else {
        return;
    };
    let config = with_stress_pool(base, clients).with_vector_index(VectorIndexType::None);
    let storage = Arc::new(PgVectorStorage::with_dimension(config, ann.dim));
    storage.initialize().await.expect("init");

    let seed_ms = seed_ws_split(
        &storage,
        ann.rows,
        ann.dim,
        ann.batch_size,
        "cap063",
        "t-cap063",
        "ws-a",
        "ws-b",
    )
    .await;

    let index_wall = Instant::now();
    storage.ensure_ann_index().await.expect("ensure_ann_index");
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    finish_report(
        "capacity_ladder_ensure_ann_index",
        &[Duration::from_secs_f64(index_ms / 1000.0)],
        600_000.0,
        "hnsw_create",
        false,
        format!(
            "ladder={} rows={} dim={} seed_ms={seed_ms:.0}",
            ladder.as_str(),
            ann.rows,
            ann.dim
        ),
    );

    let mf = workspace_filter("ws-a", "t-cap063");
    let (single_p95, single) = measure_single(&storage, ann.dim, &mf, TOP_K).await;
    let slo_pass = single_p95 < Q1D_SLO_MS;
    let under_cliff = single_p95 < cliff;
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "capacity_ladder_filtered_ann_single",
            "p95_ms": single_p95,
            "samples_ms": single.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
            "plan_class": "hnsw",
            "pass": slo_pass,
            "detail": format!(
                "ladder={} rows={} dim={} pool={pool} q1d_slo_ms={Q1D_SLO_MS} cliff_ms={cliff} slo_pass={slo_pass}",
                ladder.as_str(),
                ann.rows,
                ann.dim
            ),
        })
    );
    assert!(
        under_cliff,
        "single p95 {single_p95:.2}ms exceeds hang cliff {cliff}ms — FORBIDDEN / host undersized"
    );
    if !slo_pass {
        eprintln!(
            "WARN SPEC-063: Q1-d SLO not met at ladder {} (p95={single_p95:.2}ms > {Q1D_SLO_MS}ms) — measured cliff; do not claim 'supported at Q1-d'",
            ladder.as_str()
        );
    }

    let qpc = ann.queries_per_client;
    let (stress_p95, all, stress_wall) = measure_stress(
        Arc::clone(&storage),
        ann.dim,
        mf,
        clients,
        qpc,
        TOP_K,
    )
    .await;
    let budget = (single_p95 * mult).max(50.0);
    let stress_pass = stress_p95 < budget;
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": "capacity_ladder_filtered_ann_stress",
            "p95_ms": stress_p95,
            "samples_ms": all.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
            "plan_class": "hnsw",
            "pass": stress_pass,
            "detail": format!(
                "ladder={} clients={clients} rows={} dim={} q/client={qpc} pool={pool} single_p95={single_p95:.2} mult={mult} budget={budget:.2} slo_pass={slo_pass} stress_pass={stress_pass} wall={stress_wall:?}",
                ladder.as_str(),
                ann.rows,
                ann.dim,
            ),
        })
    );
    // Hang guard only — stress SLO miss is a measured cliff (archive + demote support claim).
    assert!(
        stress_p95 < cliff * 4.0,
        "stress p95 {stress_p95:.2}ms looks hung (>{:.0}ms)",
        cliff * 4.0
    );
    if !stress_pass {
        eprintln!(
            "WARN SPEC-063: concurrent stress not ≤{mult}× single at ladder {} (p95={stress_p95:.2} budget={budget:.2}) — measured cliff",
            ladder.as_str()
        );
    }

    eprintln!(
        "OK SPEC-063 capacity ladder {} measured: rows={} single_p95={single_p95:.2}ms stress_p95={stress_p95:.2}ms slo_pass={slo_pass} stress_pass={stress_pass} index_ms={index_ms:.0} seed_ms={seed_ms:.0}",
        ladder.as_str(),
        ann.rows
    );
    let _ = storage.clear().await;
}
