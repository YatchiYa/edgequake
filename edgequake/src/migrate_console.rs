//! Operator-facing stdout for `edgequake migrate` (SPEC-090 F-090-20b).
//!
//! Progress must be visible without `RUST_LOG` — keep `tracing` for structured logs.
//!
//! First principles (expand → migrate → contract):
//! - **Safe schema** (expand): add tables/indexes the binary needs — apply automatically.
//! - **Destroy-data** (contract): DROP old stores — never silent; needs `--confirm-drop`
//!   and a GREEN readiness check (data already copied to the new store).

#[cfg(feature = "postgres")]
use sqlx::PgPool;

/// Banner + redacted database URL.
pub fn print_banner(version: &str, redacted_database_url: &str) {
    println!("EdgeQuake migrate v{version}");
    println!("database: {redacted_database_url}");
}

/// Plain-language map of what this CLI does (printed once on apply / dry-run).
pub fn print_first_principles() {
    println!();
    println!(" WHAT THIS COMMAND DOES (first principles)");
    println!("  1. SAFE SCHEMA  — create/alter tables this binary needs.");
    println!("                   Applied now. No data deleted.");
    println!("  2. DATA MOVE    — background backfills copy old → new stores.");
    println!("                   Watched via: edgequake migrate status / dry-run.");
    println!("  3. DROP OLD     — delete legacy tables after copy is proven.");
    println!("                   NEVER automatic. Needs --confirm-drop + GREEN readiness.");
    println!("  Server start never applies migrations; it only refuses when SAFE SCHEMA");
    println!("  is still missing. Optional DROP OLD may stay pending while you serve.");
    println!();
}

/// SPEC-091: irreversible KV-store drop version (LD-07 / Wave D).
pub const KV_DROP_MIGRATION: i64 = 125;

/// SPEC-091 W4: irreversible legacy chunk-vector retirement version.
pub const VECTOR_DROP_MIGRATION: i64 = 126;

/// SPEC-091 IW2: irreversible full legacy vector fleet drop.
pub const FLEET_VECTOR_DROP_MIGRATION: i64 = 131;

/// Human one-liner for an irreversible drop version.
pub fn irreversible_drop_plain(version: i64) -> &'static str {
    match version {
        KV_DROP_MIGRATION => "deletes the old key-value tables (eq_*_kv)",
        VECTOR_DROP_MIGRATION => "deletes old chunk rows from eq_*_vectors",
        FLEET_VECTOR_DROP_MIGRATION => "deletes all remaining eq_*_vectors tables",
        _ => "irreversible drop",
    }
}

/// List pending migrations before apply.
pub fn print_preflight(pending: &[(i64, String)]) {
    println!("preflight: {} pending migration(s)", pending.len());
    if pending.is_empty() {
        println!("  (schema up to date — reconcile / post-hooks still run)");
        return;
    }
    for (version, description) in pending {
        let tag = migration_class_tag(*version);
        println!("  pending {version} — {description}{tag}");
        if is_irreversible_drop_version(*version) {
            println!("           → {}", irreversible_drop_plain(*version));
        }
    }
}

fn is_irreversible_drop_version(version: i64) -> bool {
    version == KV_DROP_MIGRATION
        || version == VECTOR_DROP_MIGRATION
        || version == FLEET_VECTOR_DROP_MIGRATION
}

/// Class label for pending-migration lines (expandable vs irreversible).
pub fn migration_class_tag(version: i64) -> &'static str {
    if version == KV_DROP_MIGRATION {
        "  [DROP OLD — irreversible KV tables]"
    } else if version == VECTOR_DROP_MIGRATION {
        "  [DROP OLD — irreversible chunk vectors]"
    } else if version == FLEET_VECTOR_DROP_MIGRATION {
        "  [DROP OLD — irreversible vector fleet]"
    } else if version == 142 {
        "  [ASSERT — SPEC-105; after confirm-drop / empty residue]"
    } else if (106..=124).contains(&version)
        || (127..=130).contains(&version)
        || (132..=141).contains(&version)
    {
        "  [SAFE SCHEMA — expandable]"
    } else {
        ""
    }
}

/// True when `version` is a SPEC-091 irreversible drop (delegates to bootstrap SSOT).
#[cfg(feature = "postgres")]
pub fn is_irreversible_drop(version: i64) -> bool {
    edgequake_api::state::migration_bootstrap::is_irreversible_drop(version)
}

#[cfg(not(feature = "postgres"))]
pub fn is_irreversible_drop(version: i64) -> bool {
    is_irreversible_drop_version(version)
}

/// All pending expandable versions (including those after a gated DROP).
#[cfg(feature = "postgres")]
pub fn pending_expandable_versions(pending: &[(i64, String)]) -> Vec<i64> {
    edgequake_api::state::migration_bootstrap::pending_expandable_versions(pending)
}

#[cfg(not(feature = "postgres"))]
pub fn pending_expandable_versions(pending: &[(i64, String)]) -> Vec<i64> {
    pending
        .iter()
        .map(|(v, _)| *v)
        .filter(|v| !is_irreversible_drop(*v))
        .collect()
}

/// Operator-facing soft-exit when only DROP OLD steps remain (safe to start server).
pub fn print_irreversible_pending_soft_exit(remaining: &[(i64, String)]) {
    eprintln!();
    eprintln!("══════════════════════════════════════════════════════════════════");
    eprintln!(" VERDICT: OK TO START THE SERVER");
    eprintln!("══════════════════════════════════════════════════════════════════");
    eprintln!(" Why: every required SAFE SCHEMA migration is applied.");
    eprintln!("      Only optional DROP OLD step(s) remain — and/or SPEC-105");
    eprintln!("      assert 142 deferred while durable legacy rows remain.");
    eprintln!("      They are NOT required for serving on typed defaults.");
    eprintln!();
    eprintln!(" Still pending (human-gated / deferred):");
    for (v, desc) in remaining {
        eprintln!("  • {v} — {desc}{}", migration_class_tag(*v));
        if is_irreversible_drop_version(*v) {
            eprintln!("      plain English: {}", irreversible_drop_plain(*v));
        } else if *v == 142 {
            eprintln!(
                "      plain English: asserts empty leftovers after confirm-drop; \
                 deferred while legacy rows remain"
            );
        }
    }
    eprintln!();
    eprintln!(" Readiness light (above):");
    eprintln!("  • GREEN = copy verified → safe to drop old tables");
    eprintln!("  • RED   = some old rows still need backfill/verify → do NOT drop yet");
    eprintln!();
    eprintln!(" When readiness is GREEN:");
    eprintln!("    1. Take a backup");
    eprintln!("    2. edgequake migrate --confirm-drop");
    eprintln!("    3. edgequake migrate            # applies deferred 142 assert");
    eprintln!(" Preview anytime: edgequake migrate dry-run");
    eprintln!(" Rollback after a drop = restore from backup (no undo SQL).");
    eprintln!("══════════════════════════════════════════════════════════════════");
}

/// Hard stop when an irreversible drop blocks later expandable schema.
pub fn print_blocked_by_irreversible(blocking: i64) {
    eprintln!();
    eprintln!("══════════════════════════════════════════════════════════════════");
    eprintln!(" VERDICT: STOP — cannot finish SAFE SCHEMA yet");
    eprintln!("══════════════════════════════════════════════════════════════════");
    eprintln!(
        " Migration {blocking} ({}) is next and blocks later SAFE SCHEMA steps.",
        irreversible_drop_plain(blocking)
    );
    eprintln!(" sqlx applies migrations in order — it cannot skip the drop.");
    eprintln!();
    eprintln!(" What to do:");
    eprintln!("  1. edgequake migrate dry-run     # see readiness");
    eprintln!("  2. When readiness is GREEN: edgequake migrate --confirm-drop");
    eprintln!("  3. Then re-run: edgequake migrate   # remaining SAFE SCHEMA");
    eprintln!("══════════════════════════════════════════════════════════════════");
}

/// Dry-run banner: preview only, zero mutations.
pub fn print_dry_run_header(version: &str, redacted_database_url: &str) {
    print_banner(version, redacted_database_url);
    println!("MODE: DRY-RUN (no changes will be applied)");
    #[cfg(feature = "postgres")]
    println!(
        "vector backend: {} (EDGEQUAKE_VECTOR_BACKEND)",
        edgequake_storage::vector_backend_from_env().as_str()
    );
    print_first_principles();
}

/// Human summary after preflight on the apply path.
pub fn print_apply_intent(pending: &[(i64, String)], drop_confirmed: bool) {
    let irreversible: Vec<i64> = pending
        .iter()
        .map(|(v, _)| *v)
        .filter(|v| is_irreversible_drop(*v))
        .collect();
    let expandable = pending
        .iter()
        .filter(|(v, _)| !is_irreversible_drop(*v))
        .count();
    println!();
    println!(" APPLY INTENT");
    println!("  pending total:              {}", pending.len());
    println!("  SAFE SCHEMA (will apply):   {expandable}");
    println!(
        "  DROP OLD (needs confirm):    {}{}",
        irreversible.len(),
        if irreversible.is_empty() {
            String::new()
        } else {
            format!(" → {irreversible:?}")
        }
    );
    if !irreversible.is_empty() {
        if drop_confirmed {
            println!("  consent: INCLUDED — --confirm-drop / fresh-install gate open");
        } else {
            println!("  consent: NOT given — will apply SAFE SCHEMA only, leave DROP OLD pending");
            println!("  tip: edgequake migrate dry-run  →  then --confirm-drop when GREEN");
        }
    }
    println!();
}

/// Operator checklist printed at the end of dry-run (and useful before apply).
pub fn print_upgrade_risk_box(has_drop_pending: bool) {
    println!();
    println!(" ╔══════════════════════════════════════════════════════════════════════╗");
    println!(" ║  UPGRADE CHECKLIST (read before applying)                            ║");
    println!(" ╚══════════════════════════════════════════════════════════════════════╝");
    println!("  1. Take a verified backup / restore point (pg_dump -Fc or volume snapshot).");
    println!("  2. Roll ALL API replicas to this write-stop binary BEFORE/with any DROP OLD.");
    println!("  3. Keep flags relational (EDGEQUAKE_CHUNK_TEXT_AUTHORITY + EDGEQUAKE_KV_FAMILY_*=relational).");
    println!("  4. Server boot never applies migrations — this CLI owns schema; boot STOPS if SAFE SCHEMA is missing.");
    if has_drop_pending {
        println!("  5. When drop-readiness is GREEN, apply DROP OLD with: edgequake migrate --confirm-drop");
        println!("     Rollback after a drop = RESTORE FROM BACKUP only.");
    } else {
        println!("  5. Apply SAFE SCHEMA with: edgequake migrate");
    }
    println!();
}

/// After a successful apply that included migration 125.
pub fn print_kv_drop_applied() {
    println!();
    println!("KV store dropped (migration {KV_DROP_MIGRATION}). Rollback = restore from backup.");
    println!();
}

/// Extra failure guidance when Wave D abort / residue blocks the drop.
pub fn print_wave_d_abort_hint(err: &dyn std::fmt::Display) {
    let msg = err.to_string();
    if msg.contains("Wave D ABORT") || msg.contains("un-migrated durable KV") {
        eprintln!("hint: residue remains outside typed SSOT.");
        eprintln!("      1) edgequake migrate dry-run");
        eprintln!("      2) edgequake migrate guard");
        eprintln!(
            "      3) finish family backfills / migration engine, then re-run --confirm-drop"
        );
    }
}

/// List versions applied in this run.
pub fn print_applied_this_run(applied: &[(i64, String)]) {
    println!("applied_this_run: {}", applied.len());
    for (version, description) in applied {
        let desc = if description.is_empty() {
            "(no description)"
        } else {
            description.as_str()
        };
        println!("  applied {version} — {desc}");
    }
}

/// Final one-line summary (machine-friendly).
pub fn print_summary(pending_before: usize, latest: Option<i64>, applied_count: usize) {
    println!(
        "migrate ok: pending_before={pending_before} latest={latest:?} applied_this_run={applied_count}"
    );
}

/// Actionable stderr hint when migrate fails.
pub fn print_failure_hint(err: &dyn std::fmt::Display) {
    eprintln!("migrate failed: {err}");
    eprintln!(
        "hint: re-run with RUST_LOG=edgequake.migration=info,edgequake=info; \
         if stuck on tasks DDL, check pg_locks / other backends holding locks on public.tasks"
    );
}

/// SPEC-091 P4: one status line per migration job (`edgequake migrate status`).
pub fn print_migration_job_line(
    step_id: &str,
    state: &str,
    processed: i64,
    estimated_total: Option<i64>,
    completion_pct: Option<f64>,
    rows_per_sec: Option<f64>,
    eta_seconds: Option<f64>,
) {
    let total = estimated_total
        .map(|t| t.to_string())
        .unwrap_or_else(|| "?".into());
    let pct = completion_pct
        .map(|p| format!("{p:>5.1}%"))
        .unwrap_or_else(|| "  ---%".into());
    let rate = rows_per_sec
        .map(|r| format!("{r:>8.1} rows/s"))
        .unwrap_or_else(|| "       ---".into());
    let eta = match eta_seconds {
        Some(s) if s >= 3600.0 => format!("ETA {:>5.1}h", s / 3600.0),
        Some(s) if s >= 60.0 => format!("ETA {:>5.1}m", s / 60.0),
        Some(s) => format!("ETA {:>5.0}s", s),
        None => "ETA    ---".into(),
    };
    println!("  {step_id:<28} {state:<10} {processed:>10}/{total:<10} {pct} {rate} {eta}");
}

/// Post-migrate schema probes (partition / PDF cutover / HNSW / partition ensure).
#[cfg(feature = "postgres")]
pub async fn print_post_hooks(pool: &PgPool) {
    match probe_tasks_partitioned(pool).await {
        Ok((partitioned, children)) => {
            if partitioned {
                println!("tasks: RANGE-partitioned (children={children})");
            } else {
                println!("tasks: not partitioned (expected after M104)");
            }
        }
        Err(e) => eprintln!("tasks partition probe failed: {e}"),
    }

    match probe_pdf_data_column(pool).await {
        Ok(present) => {
            if present {
                println!("pdf_documents.pdf_data: present (M105 not applied)");
            } else {
                println!("pdf_documents.pdf_data: absent (blob side-table SSOT)");
            }
        }
        Err(e) => eprintln!("pdf_data column probe failed: {e}"),
    }

    match edgequake_storage::check_hnsw_index_manifest(pool).await {
        Ok(drifts) => {
            println!("hnsw_manifest: drift_count={}", drifts.len());
            for d in drifts.iter().take(5) {
                println!(
                    "  drift {} expected m={} ef={} found m={:?} ef={:?}",
                    d.index_name, d.expected_m, d.expected_ef, d.found_m, d.found_ef
                );
            }
            if drifts.len() > 5 {
                println!("  … {} more", drifts.len() - 5);
            }
        }
        Err(e) => eprintln!("hnsw manifest check failed: {e}"),
    }

    match sqlx::query("SELECT edgequake_ensure_tasks_month_partitions()")
        .execute(pool)
        .await
    {
        Ok(_) => println!("tasks partitions: ensure_month_partitions ok"),
        Err(e) => {
            // Function missing on pre-M104 DBs is non-fatal for the probe line.
            eprintln!("tasks partitions: ensure_month_partitions skipped ({e})");
        }
    }
}

#[cfg(feature = "postgres")]
async fn probe_tasks_partitioned(pool: &PgPool) -> Result<(bool, i64), sqlx::Error> {
    let partitioned: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM pg_partitioned_table
          WHERE partrelid = 'public.tasks'::regclass
        )
        "#,
    )
    .fetch_one(pool)
    .await?;
    if !partitioned {
        return Ok((false, 0));
    }
    let children: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM pg_inherits i
        JOIN pg_class c ON c.oid = i.inhrelid
        JOIN pg_class p ON p.oid = i.inhparent
        WHERE p.relname = 'tasks'
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok((true, children))
}

#[cfg(feature = "postgres")]
async fn probe_pdf_data_column(pool: &PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
          SELECT 1 FROM information_schema.columns
          WHERE table_schema = 'public'
            AND table_name = 'pdf_documents'
            AND column_name = 'pdf_data'
        )
        "#,
    )
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------------------
// SPEC-091 Migration Console (doc 15 §7) — renderers for the Advisor output.
// These are pure formatters over the derived posture / guidance / actions;
// they never touch the database (the advisor already derived everything).
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres")]
use edgequake_storage::migration_engine::advisor::{
    FamilyPosture, GuardedAction, Guidance, MigrationPosture, ResidueReport,
};

/// Console banner: version, redacted URL, and the headline posture line.
#[cfg(feature = "postgres")]
pub fn print_console_banner(
    version: &str,
    redacted_database_url: &str,
    posture: &MigrationPosture,
) {
    println!("EdgeQuake migrate console v{version}");
    println!("database: {redacted_database_url}");
    let engine = format!("{:?}", posture.engine_mode).to_lowercase();
    println!(
        "cutover phase: {:<18} engine: {:<11} serving-fence: {}",
        posture.cutover_phase.as_str(),
        engine,
        if posture.serving_fence_on {
            "on"
        } else {
            "off"
        },
    );
}

/// Per-family posture table (doc 15 §7.1).
#[cfg(feature = "postgres")]
pub fn print_family_table(posture: &MigrationPosture) {
    println!();
    println!(
        " {:<22} {:<10} {:<14} {:<22} {:<14} {:>8} {:>10}",
        "FAMILY", "MODE", "PHASE", "BACKFILL", "VERIFY", "RESIDUE", "TYPED"
    );
    for f in &posture.families {
        println!(" {}", family_row(f));
    }
    // SPEC-091 W3: chunk-embedding VECTOR cutover row (separate from KV
    // families — vectors live in eq_*_vectors, not the KV store).
    println!(" {}", vector_row(posture));
}

#[cfg(feature = "postgres")]
fn vector_row(posture: &MigrationPosture) -> String {
    let v = &posture.vector;
    let phase = if v.dropped {
        "Dropped"
    } else if v.retirable() {
        // Legacy fleet fully mirrored + verified + backend flipped → safe to drop.
        "ReadyToRetire"
    } else if v.backend == "chunk_embeddings" {
        "Flipped"
    } else if v.ready_to_flip() {
        "ReadyToFlip"
    } else if v
        .backfill
        .as_ref()
        .map(|j| j.is_active() || j.state == "verifying")
        .unwrap_or(false)
    {
        "Backfilling"
    } else {
        "NotStarted"
    };
    let backfill = match &v.backfill {
        Some(j) if j.is_active() || j.state == "verifying" => j
            .completion_pct
            .map(|p| format!("{p:.1}%"))
            .unwrap_or_else(|| j.state.clone()),
        Some(j) => j.state.clone(),
        None => "—".to_string(),
    };
    let verify = match &v.verify {
        Some(x) if x.mismatches > 0 => format!("{} MISMATCH", x.mismatches),
        Some(x) if x.sampled > 0 => format!("ok ({} smpl)", x.sampled),
        Some(_) => "ok".to_string(),
        None => "—".to_string(),
    };
    format!(
        "{:<22} {:<10} {:<14} {:<22} {:<14} {:>8} {:>10}",
        "VECTOR (chunk)", v.backend, phase, backfill, verify, v.legacy_chunk_rows, v.typed_rows,
    )
}

#[cfg(feature = "postgres")]
fn family_row(f: &FamilyPosture) -> String {
    let backfill = match &f.backfill {
        Some(j) if j.is_active() || j.state == "verifying" => {
            let head = j
                .completion_pct
                .map(|p| format!("{p:.1}%"))
                .unwrap_or_else(|| j.state.clone());
            match j.eta_seconds {
                Some(s) => format!("{head} ETA {}", fmt_hms(s)),
                None => head,
            }
        }
        // Terminal jobs (completed/failed/cancelled): show the state, not 0.0%.
        Some(j) => j.state.clone(),
        None => "—".to_string(),
    };
    let verify = match &f.verify {
        Some(v) if v.mismatches > 0 => format!("{} MISMATCH", v.mismatches),
        Some(v) if v.sampled > 0 => format!("ok ({} smpl)", v.sampled),
        Some(_) => "ok".to_string(),
        None => "—".to_string(),
    };
    format!(
        "{:<22} {:<10} {:<14} {:<22} {:<14} {:>8} {:>10}",
        f.family,
        f.mode.as_str(),
        f.phase.as_str(),
        backfill,
        verify,
        f.kv_residue_rows,
        f.typed_rows,
    )
}

/// The ordered runbook (`migrate plan`, and the NEXT section of `console`).
#[cfg(feature = "postgres")]
pub fn print_instructions(guidance: &Guidance) {
    println!();
    println!(" NEXT (runbook)");
    if guidance.instructions.is_empty() {
        println!("  (no instructions — posture is stable)");
    }
    for ins in &guidance.instructions {
        println!(
            "  {:>2}. {:<8} {}",
            ins.ordinal,
            ins.kind.as_str(),
            ins.summary
        );
        if let Some(cmd) = &ins.command {
            println!("          $ {cmd}");
        }
    }
}

/// The gated action plane (doc 15 §6).
#[cfg(feature = "postgres")]
pub fn print_actions(actions: &[GuardedAction]) {
    println!();
    println!(" ACTIONS (gated)");
    if actions.is_empty() {
        println!("  (no actions available in this posture)");
    }
    for a in actions {
        let mark = if a.enabled { "✓" } else { "✗" };
        match (a.enabled, &a.gate_reason) {
            (true, _) => println!("  {mark} {} {}{}", a.verb, a.target, confirm_tag(a)),
            (false, Some(reason)) => println!("  {mark} {} {} — {}", a.verb, a.target, reason),
            (false, None) => println!("  {mark} {} {}", a.verb, a.target),
        }
    }
}

#[cfg(feature = "postgres")]
fn confirm_tag(a: &GuardedAction) -> &'static str {
    if a.requires_confirmation && a.irreversible {
        "  [requires --confirm, IRREVERSIBLE]"
    } else if a.requires_confirmation {
        "  [requires --confirm]"
    } else {
        ""
    }
}

/// `migrate guard` — read-only readiness probe with evidence (never mutates).
#[cfg(feature = "postgres")]
pub fn print_guard(posture: &MigrationPosture, residue: &ResidueReport) {
    println!();
    println!(" GUARD (read-only readiness probe)");
    println!(
        "  kv_store: {}",
        if posture.kv_store_dropped {
            "DROPPED (migration 125 applied / no eq_*_kv remain)"
        } else {
            "present"
        }
    );
    println!(
        "  durable residue: {} row(s) outside typed SSOT{}",
        residue.total(),
        if residue.total() == 0 {
            ""
        } else {
            // Show the per-category breakdown only when it blocks.
            "  <-- BLOCKS the drop"
        }
    );
    if residue.total() > 0 {
        println!("    breakdown: {}", residue.breakdown());
    }
    // Stale-flag warnings (EC-C1): any family still pointing at KV post-drop.
    for f in posture
        .families
        .iter()
        .filter(|f| posture.kv_store_dropped && f.mode.writes_kv())
    {
        println!(
            "  ✗ STALE FLAG: {}={} with KV dropped — set {}=relational (else 42P01)",
            f.env_flag,
            f.mode.as_str(),
            f.env_flag
        );
    }
    let ready = posture.global_ready_to_drop();
    println!(
        "  drop-readiness: {}",
        if ready {
            "GREEN (all families relational + 0 residue) — apply 125 with --confirm-drop"
        } else if posture.kv_store_dropped {
            "already dropped"
        } else {
            "RED (see residue / family modes above)"
        }
    );

    // SPEC-091 W4 / IW2 — readiness to DROP OLD vector tables.
    let v = &posture.vector;
    println!(
        "  vector-legacy drop-readiness: {}",
        if v.dropped {
            "already dropped — nothing left to delete".to_string()
        } else if v.retirable() {
            "GREEN — old vector rows are copied + verified; safe to --confirm-drop".to_string()
        } else {
            format!(
                "RED — do NOT drop yet (backend={}, {} old chunk row(s) still need \
                 backfill/verify; verify={})",
                v.backend,
                v.legacy_chunk_rows,
                if v.verify.map(|x| x.passes()).unwrap_or(false) {
                    "pass"
                } else {
                    "fail/pending"
                }
            )
        }
    );
    if !v.dropped && !v.retirable() {
        println!(
            "    plain English: typed tables are in use, but {} row(s) still live only",
            v.legacy_chunk_rows
        );
        println!("    in the old eq_*_vectors tables. Dropping now would lose that data.");
        println!("    Wait for backfill (edgequake migrate status) until this light is GREEN.");
    }
}

#[cfg(feature = "postgres")]
fn fmt_hms(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

#[cfg(test)]
mod first_principles_tests {
    use super::*;

    #[test]
    fn class_tags_and_plain_english() {
        assert!(irreversible_drop_plain(131).contains("eq_*_vectors"));
        assert!(irreversible_drop_plain(125).contains("key-value"));
        assert!(migration_class_tag(131).contains("DROP OLD"));
        assert!(migration_class_tag(130).contains("SAFE SCHEMA"));
    }

    #[test]
    fn apply_intent_and_verdicts_do_not_panic() {
        let pending = vec![
            (130, "fleet embeddings".into()),
            (131, "fleet vector drop".into()),
        ];
        print_apply_intent(&pending, false);
        print_first_principles();
        print_irreversible_pending_soft_exit(&[(131, "fleet vector drop".into())]);
        print_blocked_by_irreversible(131);
    }
}
