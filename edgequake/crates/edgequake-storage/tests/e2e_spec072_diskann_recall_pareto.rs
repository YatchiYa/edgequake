//! SPEC-072 — DiskANN recall×latency Pareto @150k (dedicated `*_ws_*`).
//!
//! Query grid: `diskann.query_search_list_size` ∈ {100,200,400,800}
//! vs high-quality DiskANN ref (search_list=1600). Rebuild arm if query-only fails.
//! Soft-fail product gates; hard-fail hang cliff.
//!
//! Env:
//! - `EQ_PARETO_ROWS` default `150000` (primary); set `250000` for SPEC-082 promote attempt
//! - Spot via `EQ_PARETO_SPOT_ROWS=100000,250000` (omit when primary is already 250k)
//! - `EQ_PARETO_SEARCH_LIST` default `100,200,400,800`
//! - `EQ_PARETO_REF_SEARCH_LIST` default `1600`
//! - `EQ_PARETO_REBUILD=1` (default) — rebuild arm if no full-green after query grid
//! - `EQ_DISKANN_SMOKE=1` → tiny corpus
#![cfg(feature = "postgres")]

#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_stress.rs"]
mod perf_stress;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{PgVectorStorage, VectorIndexType, VectorStorageMode};
use perf_ann_corpus::{emb, seed_single_ws};
use perf_stress::{ceiling_hang_cliff_ms, with_stress_pool};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const TOP_K: usize = 20;
const Q1D_SLO_MS: f64 = 500.0;
const RECALL_GATE: f64 = 0.99;
const DIM: usize = 1536;
const WS: &str = "ws-pareto072";
const TENANT: &str = "t-pareto072";
const GATE_CLIENTS: usize = 16;
const STRESS_ITERS: usize = 20;
const SINGLE_SAMPLES: usize = 30;

#[derive(Clone, Debug)]
struct BuildParams {
    label: &'static str,
    num_neighbors: u32,
    search_list_size: u32,
    storage_layout: &'static str,
}

const BUILD_DEFAULT: BuildParams = BuildParams {
    label: "default_sbq",
    num_neighbors: 50,
    search_list_size: 100,
    storage_layout: "memory_optimized",
};

const BUILD_HQ: BuildParams = BuildParams {
    label: "hq_n64_s200",
    num_neighbors: 64,
    search_list_size: 200,
    storage_layout: "memory_optimized",
};

fn parse_u32_list(env: &str, default: &[u32]) -> Vec<u32> {
    match std::env::var(env) {
        Ok(s) if !s.trim().is_empty() => {
            s.split(',').filter_map(|p| p.trim().parse().ok()).collect()
        }
        _ => default.to_vec(),
    }
}

fn recall_at_k(reference: &[String], candidate: &[String]) -> f64 {
    if reference.is_empty() {
        return 1.0;
    }
    let set: std::collections::HashSet<&String> = candidate.iter().collect();
    let hit = reference.iter().filter(|id| set.contains(id)).count();
    hit as f64 / reference.len() as f64
}

fn p95_ms(samples: &[Duration]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut v: Vec<f64> = samples.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64) * 0.95).ceil() as usize - 1;
    v[idx.min(v.len() - 1)]
}

fn emit(
    op: &str,
    p95_ms_v: f64,
    pass: bool,
    plan_class: &str,
    detail: impl Into<String>,
    samples: &[Duration],
) {
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "pg_major": std::env::var("EQ_POSTGRES_MAJOR").unwrap_or_default(),
            "op": op,
            "p95_ms": p95_ms_v,
            "samples_ms": samples.iter().map(|d| d.as_secs_f64() * 1000.0).collect::<Vec<_>>(),
            "plan_class": plan_class,
            "pass": pass,
            "detail": detail.into(),
        })
    );
}

async fn ensure_vectorscale(pool: &sqlx::PgPool) -> Result<(), String> {
    let ok: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_available_extensions WHERE name = 'vectorscale')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    if !ok {
        return Err("vectorscale not available — use EQ_POSTGRES_PROFILE=pg18-vectorscale".into());
    }
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE")
        .execute(pool)
        .await
        .map_err(|e| format!("CREATE EXTENSION vectorscale: {e}"))?;
    Ok(())
}

fn emb_literal(q: &[f32]) -> String {
    format!(
        "[{}]",
        q.iter()
            .map(|v| format!("{v:.8}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// DiskANN top-k with session LOCAL GUCs (ref = high search_list).
async fn topk_ids(
    pool: &sqlx::PgPool,
    table: &str,
    emb: &str,
    search_list: u32,
    rescore: u32,
) -> (Vec<String>, Duration) {
    let t0 = Instant::now();
    let mut tx = pool.begin().await.expect("topk tx");
    for stmt in edgequake_storage::diskann_query_tuning_statements(search_list, rescore) {
        let _ = sqlx::query(&stmt).execute(&mut *tx).await;
    }
    // Dedicated `*_ws_*` table is already single-workspace; omit JSONB filters so the
    // planner picks DiskANN Index Scan (filters → Seq+Sort even with enable_seqscan=off
    // on some stats shapes). Corpus seed is one WS/tenant/type.
    let _ = sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *tx)
        .await;
    let _ = sqlx::query("SET LOCAL random_page_cost = 1.1")
        .execute(&mut *tx)
        .await;
    let sql = format!(
        r#"SELECT id FROM {table}
           ORDER BY embedding <=> $1::vector
           LIMIT {TOP_K}"#
    );
    let rows: Vec<(String,)> = sqlx::query_as(&sql)
        .bind(emb)
        .fetch_all(&mut *tx)
        .await
        .unwrap_or_default();
    let _ = tx.commit().await;
    (rows.into_iter().map(|r| r.0).collect(), t0.elapsed())
}

async fn measure_recall(
    pool: &sqlx::PgPool,
    table: &str,
    cand_list: u32,
    cand_rescore: u32,
    ref_list: u32,
    ref_rescore: u32,
) -> f64 {
    let mut recalls = Vec::new();
    for s in 0..5 {
        let q = emb(DIM, (s + 42) as f32);
        let lit = emb_literal(&q);
        let (ref_ids, _) = topk_ids(pool, table, &lit, ref_list, ref_rescore).await;
        let (cand_ids, _) = topk_ids(pool, table, &lit, cand_list, cand_rescore).await;
        recalls.push(recall_at_k(&ref_ids, &cand_ids));
    }
    recalls.iter().sum::<f64>() / recalls.len() as f64
}

async fn measure_single_diskann(
    pool: &sqlx::PgPool,
    table: &str,
    search_list: u32,
    rescore: u32,
) -> (f64, Vec<Duration>) {
    let mut samples = Vec::with_capacity(SINGLE_SAMPLES);
    for s in 0..SINGLE_SAMPLES {
        let q = emb(DIM, (s as f32) * 1.7 + 3.0);
        let lit = emb_literal(&q);
        let (_, d) = topk_ids(pool, table, &lit, search_list, rescore).await;
        samples.push(d);
    }
    (p95_ms(&samples), samples)
}

async fn measure_stress_diskann(
    pool: Arc<sqlx::PgPool>,
    table: String,
    search_list: u32,
    rescore: u32,
    clients: usize,
) -> (f64, Vec<Duration>, Duration) {
    let sem = Arc::new(Semaphore::new(clients));
    let wall = Instant::now();
    let mut handles = Vec::new();
    for i in 0..(clients * STRESS_ITERS) {
        let permit = sem.clone().acquire_owned().await.expect("sem");
        let pool = Arc::clone(&pool);
        let table = table.clone();
        handles.push(tokio::spawn(async move {
            let _p = permit;
            let q = emb(DIM, (i as f32) * 0.37 + 11.0);
            let lit = emb_literal(&q);
            let (_, d) = topk_ids(&pool, &table, &lit, search_list, rescore).await;
            d
        }));
    }
    let mut all = Vec::new();
    for h in handles {
        all.push(h.await.expect("join"));
    }
    let wall_d = wall.elapsed();
    (p95_ms(&all), all, wall_d)
}

struct Seeded {
    pool: Arc<sqlx::PgPool>,
    table: String,
    index_name: String,
    #[allow(dead_code)]
    storage: Arc<PgVectorStorage>,
}

async fn seed_diskann(rows: usize, build: &BuildParams) -> Result<Seeded, String> {
    let base = postgres_test_config::require_or_skip_postgres("pareto072")
        .ok_or_else(|| "DATABASE_URL required".to_string())?;
    let mut config = with_stress_pool(base, GATE_CLIENTS).with_vector_index(VectorIndexType::None);
    config.namespace = format!(
        "eq_p072_ws_{}_{}",
        build.label,
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    let probe = postgres_test_config::contract_pg_pool(&config).await;
    ensure_vectorscale(&probe).await?;

    let storage = Arc::new(
        PgVectorStorage::with_dimension(config.clone(), DIM)
            .with_storage_mode(VectorStorageMode::Full),
    );
    storage.initialize().await.map_err(|e| e.to_string())?;
    assert!(storage.is_dedicated_workspace_table());

    let seed_ms = seed_single_ws(&storage, rows, DIM, 1000, "p072", TENANT, WS).await;
    let table = storage.vectors_table_name().to_string();
    let table_only = table.rsplit('.').next().unwrap_or(&table);
    let idx = format!("{table_only}_diskann_idx");
    let index_wall = Instant::now();
    sqlx::query(&format!(
        r#"CREATE INDEX {idx} ON {table}
           USING diskann (embedding vector_cosine_ops)
           WITH (
             storage_layout = '{layout}',
             num_neighbors = {nn},
             search_list_size = {sls}
           )"#,
        layout = build.storage_layout,
        nn = build.num_neighbors,
        sls = build.search_list_size,
    ))
    .execute(&probe)
    .await
    .map_err(|e| format!("CREATE INDEX diskann: {e}"))?;
    let _ = sqlx::query(&format!("ANALYZE {table}"))
        .execute(&probe)
        .await;
    // Confirm DiskANN index is visible to the planner.
    let idx_ok: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS(SELECT 1 FROM pg_indexes WHERE indexname = '{idx}' AND indexdef ILIKE '%diskann%')"
    ))
    .fetch_one(&probe)
    .await
    .unwrap_or(false);
    if !idx_ok {
        return Err(format!("diskann index missing after CREATE: {idx}"));
    }
    let index_ms = index_wall.elapsed().as_secs_f64() * 1000.0;
    emit(
        "pareto_index",
        index_ms,
        true,
        build.label,
        format!(
            "build={} rows={rows} seed_ms={seed_ms:.0} nn={} sls={} table={table}",
            build.label, build.num_neighbors, build.search_list_size
        ),
        &[Duration::from_secs_f64(index_ms / 1000.0)],
    );

    // EXPLAIN sanity
    let emb: String = {
        let vals: Vec<String> = (0..DIM)
            .map(|i| format!("{:.8}", ((i as f32 + 10.0) * 0.019).sin()))
            .collect();
        format!("[{}]", vals.join(","))
    };
    let mut tx = probe.begin().await.expect("explain");
    let _ = sqlx::query("SET LOCAL diskann.query_search_list_size = 100")
        .execute(&mut *tx)
        .await;
    let _ = sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *tx)
        .await;
    // Unfiltered ORDER BY — dedicated table corpus is single-WS (matches measure path).
    let plan_rows: Vec<(String,)> = sqlx::query_as(&format!(
        r#"EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
           SELECT id FROM {table}
           ORDER BY embedding <=> $1::vector LIMIT 20"#
    ))
    .bind(&emb)
    .fetch_all(&mut *tx)
    .await
    .unwrap_or_default();
    let _ = tx.commit().await;
    let plan = plan_rows
        .into_iter()
        .map(|r| r.0)
        .collect::<Vec<_>>()
        .join("\n");
    // pgvectorscale plans show "Index Scan using <name>" — not the token "diskann".
    let uses_diskann = plan.to_lowercase().contains(&idx.to_lowercase())
        || (plan.to_lowercase().contains("index scan")
            && !plan.to_lowercase().contains("->  sort"));
    emit(
        "pareto_explain",
        0.0,
        uses_diskann,
        build.label,
        format!(
            "unfiltered_dedicated_ws_shape\n{}",
            plan.chars().take(2800).collect::<String>()
        ),
        &[],
    );
    if !uses_diskann {
        return Err(format!(
            "planner not using diskann index {idx} (sort/seq path); plan_head={}",
            plan.chars().take(200).collect::<String>()
        ));
    }

    let _ = config;
    Ok(Seeded {
        pool: Arc::new(probe),
        table,
        index_name: idx,
        storage,
    })
}

async fn drop_index(pool: &sqlx::PgPool, index_name: &str) {
    let _ = sqlx::query(&format!("DROP INDEX IF EXISTS {index_name}"))
        .execute(pool)
        .await;
}

struct Cell {
    full_green: bool,
    recall_ok: bool,
    abs_ok: bool,
    recall: f64,
    single_p95: f64,
    stress_p95: f64,
}

#[allow(clippy::too_many_arguments)] // diskann pareto cell knobs stay explicit for emit
async fn run_query_cell(
    seeded: &Seeded,
    rows: u32,
    build_label: &str,
    search_list: u32,
    rescore: u32,
    ref_list: u32,
    ref_rescore: u32,
    cliff: f64,
) -> Cell {
    let arm = format!("diskann_q{search_list}_r{rescore}_{build_label}");
    let recall = measure_recall(
        &seeded.pool,
        &seeded.table,
        search_list,
        rescore,
        ref_list,
        ref_rescore,
    )
    .await;
    let recall_ok = recall >= RECALL_GATE;
    emit(
        "pareto_recall",
        recall * 1000.0,
        recall_ok,
        &arm,
        format!(
            "rows={rows} build={build_label} q_list={search_list} q_rescore={rescore} \
             ref_list={ref_list} recall@20={recall:.4} gate={RECALL_GATE} \
             ref=high_diskann_search_list"
        ),
        &[],
    );

    let (single_p95, single) =
        measure_single_diskann(&seeded.pool, &seeded.table, search_list, rescore).await;
    let slo_pass = single_p95 < Q1D_SLO_MS;
    assert!(
        single_p95 < cliff,
        "hang cliff: {arm} single {single_p95:.2} >= {cliff}"
    );
    emit(
        "pareto_single",
        single_p95,
        slo_pass && recall_ok,
        &arm,
        format!("rows={rows} build={build_label} q_list={search_list} slo_pass={slo_pass}"),
        &single,
    );

    let (stress_p95, all, wall) = measure_stress_diskann(
        Arc::clone(&seeded.pool),
        seeded.table.clone(),
        search_list,
        rescore,
        GATE_CLIENTS,
    )
    .await;
    let abs_ok = stress_p95 < Q1D_SLO_MS;
    let full_green = slo_pass && recall_ok && abs_ok;
    emit(
        "pareto_stress",
        stress_p95,
        full_green,
        &arm,
        format!(
            "rows={rows} build={build_label} q_list={search_list} clients={GATE_CLIENTS} \
             single_p95={single_p95:.2} abs_ok={abs_ok} wall={wall:?} full_green={full_green}"
        ),
        &all,
    );
    emit(
        "pareto_cell",
        stress_p95,
        full_green,
        &arm,
        format!(
            "rows={rows} build={build_label} q_list={search_list} q_rescore={rescore} \
             recall={recall:.4} single_p95={single_p95:.2} stress_p95={stress_p95:.2} \
             full_green={full_green}"
        ),
        &[],
    );
    Cell {
        full_green,
        recall_ok,
        abs_ok,
        recall,
        single_p95,
        stress_p95,
    }
}

async fn run_grid_on_seed(
    seeded: &Seeded,
    rows: u32,
    build: &BuildParams,
    search_lists: &[u32],
    ref_list: u32,
    cliff: f64,
) -> bool {
    let mut any_green = false;
    let ref_rescore = (ref_list / 2).max(100);
    for &q_list in search_lists {
        let rescore = (q_list / 2).max(50);
        let cell = run_query_cell(
            seeded,
            rows,
            build.label,
            q_list,
            rescore,
            ref_list,
            ref_rescore,
            cliff,
        )
        .await;
        if cell.full_green {
            any_green = true;
            eprintln!(
                "GREEN SPEC-072: rows={rows} build={} q_list={q_list} recall={:.4} \
                 single={:.2} stress={:.2}",
                build.label, cell.recall, cell.single_p95, cell.stress_p95
            );
        } else {
            eprintln!(
                "NOTE SPEC-072: not green rows={rows} build={} q_list={q_list} \
                 recall_ok={} abs_ok={} recall={:.4}",
                build.label, cell.recall_ok, cell.abs_ok, cell.recall
            );
        }
    }
    any_green
}

#[tokio::test]
async fn e2e_spec072_diskann_recall_pareto() {
    let smoke = std::env::var("EQ_DISKANN_SMOKE")
        .map(|v| v == "1")
        .unwrap_or(false);
    // Smoke forces tiny corpus even if runner exported EQ_PARETO_ROWS=150000.
    let primary = if smoke {
        2_000
    } else {
        parse_u32_list("EQ_PARETO_ROWS", &[150_000])[0]
    };
    let spot = if smoke {
        vec![]
    } else {
        parse_u32_list("EQ_PARETO_SPOT_ROWS", &[100_000, 250_000])
    };
    let search_lists = parse_u32_list("EQ_PARETO_SEARCH_LIST", &[100, 200, 400, 800]);
    let ref_list = parse_u32_list("EQ_PARETO_REF_SEARCH_LIST", &[1_600])[0];
    let do_rebuild = std::env::var("EQ_PARETO_REBUILD")
        .map(|v| v != "0")
        .unwrap_or(true);

    std::env::remove_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE");
    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "full");

    if postgres_test_config::require_or_skip_postgres("pareto072").is_none() {
        return;
    }

    let mut green_150k = false;
    let mut best_detail = String::from("none");

    // --- Primary @150k (or smoke) ---
    let cliff = ceiling_hang_cliff_ms(primary as usize);
    let seeded = match seed_diskann(primary as usize, &BUILD_DEFAULT).await {
        Ok(s) => s,
        Err(e) => {
            emit(
                "pareto_decision",
                0.0,
                false,
                "promote",
                format!("ERR seed default: {e}"),
                &[],
            );
            return;
        }
    };
    let green_default = run_grid_on_seed(
        &seeded,
        primary,
        &BUILD_DEFAULT,
        &search_lists,
        ref_list,
        cliff,
    )
    .await;
    let mut green_250k = false;
    if green_default && primary == 150_000 {
        green_150k = true;
        best_detail = format!("build={} query_grid_green", BUILD_DEFAULT.label);
    }
    if green_default && primary == 250_000 {
        green_250k = true;
        best_detail = format!("build={} query_grid_green rows=250000", BUILD_DEFAULT.label);
    }

    if !green_default && do_rebuild && !smoke {
        eprintln!(
            "NOTE SPEC-072: query-only failed — rebuild arm {}",
            BUILD_HQ.label
        );
        drop_index(&seeded.pool, &seeded.index_name).await;
        match seed_diskann(primary as usize, &BUILD_HQ).await {
            Ok(hq) => {
                let green_hq =
                    run_grid_on_seed(&hq, primary, &BUILD_HQ, &search_lists, ref_list, cliff).await;
                if green_hq && primary == 150_000 {
                    green_150k = true;
                    best_detail = format!("build={} query_grid_green", BUILD_HQ.label);
                }
                if green_hq && primary == 250_000 {
                    green_250k = true;
                    best_detail = format!("build={} query_grid_green rows=250000", BUILD_HQ.label);
                }
                emit(
                    "pareto_rebuild",
                    0.0,
                    green_hq,
                    BUILD_HQ.label,
                    format!("rebuild_full_green={green_hq} rows={primary}"),
                    &[],
                );
            }
            Err(e) => emit(
                "pareto_rebuild",
                0.0,
                false,
                BUILD_HQ.label,
                format!("ERR: {e}"),
                &[],
            ),
        }
    }

    // --- Spot checks (best-effort: default build, mid search_list) ---
    for &rows in &spot {
        if rows == primary {
            continue;
        }
        let cliff_s = ceiling_hang_cliff_ms(rows as usize);
        match seed_diskann(rows as usize, &BUILD_DEFAULT).await {
            Ok(s) => {
                // Spot: q_list=400 (mid) + 800 (high) only
                let spot_lists: Vec<u32> =
                    search_lists.iter().copied().filter(|v| *v >= 400).collect();
                let lists = if spot_lists.is_empty() {
                    search_lists.clone()
                } else {
                    spot_lists
                };
                let g = run_grid_on_seed(&s, rows, &BUILD_DEFAULT, &lists, ref_list, cliff_s).await;
                emit(
                    "pareto_spot",
                    0.0,
                    g,
                    "spot",
                    format!("rows={rows} any_full_green={g}"),
                    &[],
                );
            }
            Err(e) => emit(
                "pareto_spot",
                0.0,
                false,
                "spot",
                format!("rows={rows} ERR: {e}"),
                &[],
            ),
        }
    }

    // SPEC-082: allow primary=250k full-gate promote (opt-in DiskANN floor push).
    let promote = (green_150k || green_250k) && !smoke;
    let highest_green = if green_250k {
        250_000
    } else if green_150k {
        150_000
    } else {
        0
    };
    emit(
        "pareto_decision",
        if promote { 1.0 } else { 0.0 },
        promote,
        "promote",
        format!(
            "green_150k={green_150k} green_250k={green_250k} highest_green_N={highest_green} \
             promote_ssot={promote} best={best_detail} \
             ref_search_list={ref_list} smoke={smoke} \
             (full gate: single∧recall@20≥0.99∧concurrent@clients=16) \
             ref_method=high_diskann_query_search_list_size"
        ),
        &[],
    );
}
