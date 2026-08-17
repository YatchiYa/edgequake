//! SPEC-091 Migration Console — e2e across every earlier cutover phase.
//!
//! Drives the advisor (`posture()` → `derive_guidance()` / `derive_actions()`)
//! through each previous-version state against a dedicated **pre-drop** fixture
//! database (migrations 001..=124 applied, 125 absent — see
//! `support/spec091_fixture.rs`), plus the genuinely-dropped dev DB for the
//! EC-C1 stale-flag regression. Also proves the advisor's durable-residue guard
//! is byte-for-byte faithful to migration 125's guard (parity test).
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test e2e_spec091_console -- --nocapture

#![cfg(feature = "postgres")]

#[path = "support/spec091_fixture.rs"]
mod fixture;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::migration_engine::advisor::{
    self, CutoverPhase, FamilyMode, FamilyPhase, InstrKind,
};
use postgres_test_config::require_or_skip_postgres;
use sqlx::PgPool;
use uuid::Uuid;

/// Connect a pool to the shared pre-drop fixture DB (provisioning it on first
/// use). Returns `None` to soft-skip when the server is unreachable.
async fn fixture_pool() -> Option<PgPool> {
    let cfg = require_or_skip_postgres("spec091_console")?;
    // Once a DB is configured, a provisioning failure is a bug (wrong path,
    // failed migration) — fail loudly rather than silently skip.
    let url = fixture::predrop_fixture_url(&cfg)
        .await
        .expect("provision pre-drop fixture DB failed (migrations replay/bookkeeping)");
    Some(PgPool::connect(&url).await.expect("fixture pool"))
}

/// Acquire the console lock (serializes env mutation + the shared fixture DB),
/// connect + reset the fixture, and clear all family flags.
macro_rules! fresh_fixture {
    () => {{
        let _guard = fixture::console_lock().lock().await;
        let Some(pool) = fixture_pool().await else {
            return;
        };
        fixture::clear_family_env();
        fixture::reset_predrop_fixture(&pool).await;
        (pool, _guard)
    }};
}

fn family_phase(posture: &advisor::MigrationPosture, name: &str) -> FamilyPhase {
    posture.family(name).unwrap().phase
}

// ---------------------------------------------------------------------------
// Fresh pre-cutover deployment: nothing migrated to typed yet.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn spec091_console_fresh_all_not_started() {
    let (pool, _guard) = fresh_fixture!();
    let table = fixture::create_kv_table(&pool, "fresh").await;
    let doc = Uuid::new_v4();
    fixture::seed_all_durable_residue(&pool, &table, doc).await;
    // All families still on KV.
    for var in fixture::ALL_FAMILY_ENV_VARS {
        std::env::set_var(var, "kv");
    }

    let posture = advisor::posture(&pool).await.expect("posture");
    assert!(!posture.kv_store_dropped, "fixture is pre-drop");
    assert_eq!(posture.cutover_phase, CutoverPhase::NotStarted);
    for durable in [
        "CHUNK",
        "METADATA",
        "WSDOC",
        "DOC_HASH",
        "ARTIFACT",
        "INJECTION",
    ] {
        assert_eq!(
            family_phase(&posture, durable),
            FamilyPhase::NotStarted,
            "{durable} should be NotStarted (kv + residue + no job)"
        );
    }

    // Runbook is non-empty and points every durable family at the remedy.
    let g = advisor::derive_guidance(&posture);
    assert!(!g.instructions.is_empty(), "runbook never empty pre-drop");
    assert!(g
        .instructions
        .iter()
        .any(|i| i.kind == InstrKind::Action && i.summary.contains("CHUNK")));

    // Drop is gated (residue everywhere + nothing relational).
    let actions = advisor::derive_actions(&posture);
    let drop = actions.iter().find(|a| a.verb == "drop").unwrap();
    assert!(!drop.enabled);
    fixture::clear_family_env();
}

// ---------------------------------------------------------------------------
// Chunk dual-writing with an in-flight backfill → WAIT with live progress.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn spec091_console_chunk_backfilling_waits() {
    let (pool, _guard) = fresh_fixture!();
    let table = fixture::create_kv_table(&pool, "backfill").await;
    let doc = Uuid::new_v4();
    fixture::seed_chunk_text_residue(&pool, &table, doc, 501).await;
    std::env::set_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY", "dual");
    fixture::seed_chunk_backfill_job(&pool, "running", 420, 1000).await;

    let posture = advisor::posture(&pool).await.expect("posture");
    assert_eq!(family_phase(&posture, "CHUNK"), FamilyPhase::Backfilling);
    let job = posture.family("CHUNK").unwrap().backfill.as_ref().unwrap();
    assert_eq!(job.state, "running");
    assert!(job.completion_pct.unwrap() > 0.0);

    // Guidance says WAIT (do not flip) and carries the live percentage.
    let g = advisor::derive_guidance(&posture);
    let wait = g
        .instructions
        .iter()
        .find(|i| i.kind == InstrKind::Wait)
        .expect("a running backfill yields a WAIT");
    assert!(wait.summary.contains("WAIT"));
    assert!(wait.summary.contains("Do not flip"));

    // Job control mirrors the lease state machine exactly (Pause/Cancel legal
    // from running; Resume is only legal from paused, so it is gated here).
    let actions = advisor::derive_actions(&posture);
    for verb in ["pause", "cancel"] {
        assert!(
            actions.iter().any(|a| a.verb == verb && a.enabled),
            "{verb} should be enabled on a running job"
        );
    }
    let resume = actions.iter().find(|a| a.verb == "resume").unwrap();
    assert!(
        !resume.enabled,
        "resume is gated from running (legal only from paused)"
    );
    let set = actions
        .iter()
        .find(|a| a.verb == "family.set relational" && a.target == "CHUNK")
        .unwrap();
    assert!(!set.enabled);
    fixture::clear_family_env();
}

// ---------------------------------------------------------------------------
// Backfill complete + verify clean → ReadyToFlip (flip enabled).
// ---------------------------------------------------------------------------
#[tokio::test]
async fn spec091_console_chunk_ready_to_flip() {
    let (pool, _guard) = fresh_fixture!();
    // KV present but fully drained (no chunk-text residue); typed side holds
    // the content so the content verify passes.
    let _table = fixture::create_kv_table(&pool, "flip").await;
    std::env::set_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY", "dual");
    fixture::seed_chunk_backfill_job(&pool, "completed", 1000, 1000).await;

    let posture = advisor::posture(&pool).await.expect("posture");
    assert_eq!(family_phase(&posture, "CHUNK"), FamilyPhase::ReadyToFlip);

    let g = advisor::derive_guidance(&posture);
    let action = g
        .instructions
        .iter()
        .find(|i| {
            i.kind == InstrKind::Action && i.summary.contains("EDGEQUAKE_CHUNK_TEXT_AUTHORITY")
        })
        .expect("ReadyToFlip yields a flip ACTION");
    assert_eq!(
        action.command.as_deref(),
        Some("export EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational")
    );

    let actions = advisor::derive_actions(&posture);
    let set = actions
        .iter()
        .find(|a| a.verb == "family.set relational" && a.target == "CHUNK")
        .unwrap();
    assert!(set.enabled, "flip is enabled once ReadyToFlip");
    fixture::clear_family_env();
}

// ---------------------------------------------------------------------------
// All relational + KV drained → ReadyToDrop (drop enabled, needs confirmation).
// ---------------------------------------------------------------------------
#[tokio::test]
async fn spec091_console_ready_to_drop() {
    let (pool, _guard) = fresh_fixture!();
    let _table = fixture::create_kv_table(&pool, "drop").await; // present but empty
    for var in fixture::ALL_FAMILY_ENV_VARS {
        std::env::set_var(var, "relational");
    }

    let posture = advisor::posture(&pool).await.expect("posture");
    assert!(!posture.kv_store_dropped);
    assert!(posture.global_ready_to_drop());
    assert_eq!(posture.cutover_phase, CutoverPhase::ReadyToDrop);
    assert_eq!(family_phase(&posture, "CHUNK"), FamilyPhase::ReadyToDrop);

    let g = advisor::derive_guidance(&posture);
    let confirm = g
        .instructions
        .iter()
        .find(|i| i.kind == InstrKind::Confirm)
        .expect("ReadyToDrop yields a CONFIRM");
    assert!(confirm.summary.contains("--confirm-drop"));
    assert!(confirm.summary.contains("IRREVERSIBLE"));

    let actions = advisor::derive_actions(&posture);
    let drop = actions.iter().find(|a| a.verb == "drop").unwrap();
    assert!(drop.enabled && drop.requires_confirmation && drop.irreversible);
    fixture::clear_family_env();
}

// ---------------------------------------------------------------------------
// EC-C1 regression: stale `dual` flag against the genuinely-dropped dev DB.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn spec091_console_dropped_stale_flag_blocks() {
    let _guard = fixture::console_lock().lock().await;
    let Some(cfg) = require_or_skip_postgres("spec091_console_drop") else {
        return;
    };
    let pool = postgres_test_config::contract_pg_pool(&cfg).await;
    // The dev DB has migration 125 applied → genuinely dropped. A stale dual
    // flag must be caught (the original 42P01 bug).
    fixture::clear_family_env();
    std::env::set_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY", "dual");

    let posture = advisor::posture(&pool).await.expect("posture");
    assert!(posture.kv_store_dropped, "dev DB is post-drop");
    assert_eq!(posture.cutover_phase, CutoverPhase::Dropped);
    assert_eq!(family_phase(&posture, "CHUNK"), FamilyPhase::Dropped);

    let g = advisor::derive_guidance(&posture);
    let blocked: Vec<_> = g
        .instructions
        .iter()
        .filter(|i| i.kind == InstrKind::Blocked)
        .collect();
    assert!(
        blocked.iter().any(|i| i.summary.contains("42P01")),
        "stale flag must surface the 42P01 warning; got {:?}",
        g.instructions
    );

    // The remedy (set→relational) is enabled; any rollback to kv stays refused.
    let actions = advisor::derive_actions(&posture);
    let remedy = actions
        .iter()
        .find(|a| a.verb == "family.set relational" && a.target == "CHUNK")
        .expect("stale dual flag offers the relational remedy");
    assert!(remedy.enabled);
    assert!(!actions
        .iter()
        .any(|a| a.verb == "family.set kv" && a.enabled));
    fixture::clear_family_env();
}

// ---------------------------------------------------------------------------
// C3 parity: the advisor's durable-residue verdict must match migration 125's
// guard exactly — on both a residue-bearing and a clean KV table (incl. EC-34,
// where chunk text present in typed `chunks` makes the KV copy redundant).
// ---------------------------------------------------------------------------

/// Extract migration 125's verified-purge DO block + durable-residue guard.
/// LAW-KVH5: advisor purge-aware residue must match `--confirm-drop` (purge then guard).
fn extract_drop_guard_sql() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../migrations/125_spec091_kv_drop.sql"
    );
    let sql = std::fs::read_to_string(path).expect("read migration 125");
    let purge_marker = "Verified purge of presence-conservative families";
    let purge_at = sql
        .find(purge_marker)
        .expect("verified purge comment in migration 125");
    let start = sql[purge_at..]
        .find("DO $$")
        .map(|i| purge_at + i)
        .expect("purge DO block start");
    let marker = "un-migrated durable KV rows";
    let marker_at = sql
        .find(marker)
        .expect("guard abort message in migration 125");
    let end = sql[marker_at..]
        .find("END $$;")
        .map(|i| marker_at + i + "END $$;".len())
        .expect("guard DO block end");
    sql[start..end].to_string()
}

/// Run the real migration-125 guard; returns Ok(()) when it passes (no durable
/// residue) or Err(msg) when it aborts.
async fn run_real_drop_guard(pool: &PgPool) -> Result<(), String> {
    sqlx::raw_sql(&extract_drop_guard_sql())
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn contract_spec091_advisor_matches_125_guard() {
    let (pool, _guard) = fresh_fixture!();

    // 1) Residue-bearing KV (chunk text NOT in typed) → both say "blocked".
    let table = fixture::create_kv_table(&pool, "parity").await;
    let orphan = Uuid::new_v4();
    fixture::seed_chunk_text_residue(&pool, &table, orphan, 3).await;

    let advisor_total = advisor::guard_durable_total(&pool, &table)
        .await
        .expect("advisor residue");
    assert_eq!(advisor_total, 3, "advisor counts 3 orphan chunk rows");
    let guard = run_real_drop_guard(&pool).await;
    assert!(
        guard.is_err() && guard.unwrap_err().contains("un-migrated durable KV rows"),
        "real guard must abort on residue"
    );

    // 2) EC-34: represent those chunks in typed `chunks` → both say "clean".
    fixture::seed_typed_document(&pool, orphan).await; // chunks.document_id FK
    for i in 0..3 {
        fixture::seed_typed_chunk(&pool, orphan, i, "chunk text").await;
    }
    let advisor_total = advisor::guard_durable_total(&pool, &table)
        .await
        .expect("advisor residue");
    assert_eq!(advisor_total, 0, "typed representation makes KV redundant");
    run_real_drop_guard(&pool)
        .await
        .expect("real guard passes once chunks are typed");

    // 3) A conservative-presence key (dedup) stays blocking even with no chunk
    // residue — the guard does not trust prefixes alone for these families.
    fixture::seed_kv_row(&pool, &table, "doc:hash:zzz", serde_json::json!({"h": 1})).await;
    let advisor_total = advisor::guard_durable_total(&pool, &table)
        .await
        .expect("advisor residue");
    assert_eq!(advisor_total, 1, "dedup presence is conservatively durable");
    assert!(run_real_drop_guard(&pool).await.is_err());
}

// ---------------------------------------------------------------------------
// C0/C2 render-free surface: `family list`-equivalent posture table is
// populated for all 10 families with sensible per-family facts.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn spec091_console_posture_covers_all_families() {
    let (pool, _guard) = fresh_fixture!();
    let _table = fixture::create_kv_table(&pool, "list").await;
    let posture = advisor::posture(&pool).await.expect("posture");
    assert_eq!(posture.families.len(), advisor::FAMILIES.len());
    for f in &posture.families {
        // typed_rows is a non-negative COUNT; env_flag is the SSOT var name.
        assert!(f.typed_rows >= 0);
        assert!(f.env_flag.starts_with("EDGEQUAKE_"));
        // durable families map to a residue bucket; transient ones to 0.
        if !f.durable {
            assert_eq!(posture.residue.for_family(f.family), 0);
        }
    }
    assert!(matches!(
        posture
            .families
            .iter()
            .find(|f| f.family == "CHUNK")
            .map(|f| f.mode),
        Some(FamilyMode::Relational) // default post-Wave-D when flag unset
    ));
}
