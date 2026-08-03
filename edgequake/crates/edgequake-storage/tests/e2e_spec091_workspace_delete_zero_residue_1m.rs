//! SPEC-091 IW5: workspace delete zero-residue scale proof.
//!
//! Default (CI): 100-row residue gate. With `EQ_SCALE_PROOF=1`: 10k rows as an
//! L3 stand-in (1M is reserved for nightly soak — see workflow schedule).
//!
//! Run:
//!   cargo test -p edgequake-storage --features postgres \
//!     --test e2e_spec091_workspace_delete_zero_residue_1m -- --test-threads=1
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};

const DIM: usize = 1536;

fn target_chunk_count() -> usize {
    if std::env::var("EQ_SCALE_PROOF")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        // L3 stand-in: nightly soak targets 1M; 10k exercises bulk delete paths.
        10_000
    } else {
        100
    }
}

#[tokio::test]
async fn e2e_spec091_workspace_delete_zero_residue() {
    let Some(cfg) = require_or_skip_postgres("spec091_residue") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;

    let n = target_chunk_count();
    eprintln!("SPEC-091 IW5 residue proof: seeding {n} chunks");

    let ws = w3::seed_workspace(&pool, "residue").await;
    let doc = w3::seed_document(&pool, ws).await;
    w3::seed_chunks_bulk(&pool, doc, ws, n).await;
    w3::seed_typed_embeddings_bulk(&pool, ws, "iw5-residue-model", DIM, 700).await;

    let (c0, e0, d0) = w3::count_workspace_residue(&pool, ws).await;
    assert_eq!(c0, n as i64);
    assert_eq!(e0, n as i64);
    assert_eq!(d0, 1);

    w3::delete_workspace_cascade(&pool, ws).await;

    let (c1, e1, d1) = w3::count_workspace_residue(&pool, ws).await;
    assert_eq!(c1, 0, "chunks residue after workspace delete");
    assert_eq!(e1, 0, "chunk_embeddings residue after workspace delete");
    assert_eq!(d1, 0, "documents residue after workspace delete");
}
