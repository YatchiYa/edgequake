# 17 — Boot Migration Gating: No Silent Auto-Migrate at Server Start

> **Status:** **IMPLEMENTED** (LD-15 / LAW-B1..B5) — 2026-07-31. Serving boot is fail-closed verify-only; `EDGEQUAKE_ALLOW_BOOT_MIGRATE` is **removed** (warn-and-ignore shim). Schema apply is `edgequake migrate` only. Downgrade refuse (LAW-B5) is live via `schema_drift`. Proven by `contract_spec091_boot_gate`. The `edgequake migrate` CLI + `dry-run` + `--confirm-drop` remain the operator path ([15-migration-console-cli.md](15-migration-console-cli.md)).
> **Scope:** the boundary between **server start** and **schema change** — who may mutate versioned schema, when, and how the operator previews it first. Out of scope: the migration engine's data backfills (LD-08, [07-migration-engine.md](07-migration-engine.md)), and additive runtime object-ensure (classified in §6, bounded, not removed).
> **Builds on:** [07-migration-engine.md](07-migration-engine.md) (one schema owner) · [06-implementation-plan.md](06-implementation-plan.md) (LD-03) · [15-migration-console-cli.md](15-migration-console-cli.md) (the CLI that becomes the sole schema writer) · [16-post-cutover-assessment.md](16-post-cutover-assessment.md)
> **Laws:** LAW-D5 (one schema owner, no runtime DDL), LAW-D8 (data movement never runs at boot), LD-03 (runtime code never issues DDL), LD-07 (flag-gated change; irreversible is explicit), LD-14 (operator guidance is schema-derived) — extended by **LAW-B1..B5** (§2) and locked as **LD-15**.

---

## 1. WHY — a server boot is not consent

A database migration is an **operator decision**: it can be irreversible (migration **125** dropped `eq_*_kv`; migration **126** retires chunk vectors), it can lock or rewrite relations, and it must be previewable before it is applied. A server process starting up is an **operational event**: it happens on every deploy, every crash-restart, every autoscaler scale-out — with no human in the loop and no preview. Today these two are conflated: setting `EDGEQUAKE_ALLOW_BOOT_MIGRATE=1` (which `make dev` does by default) lets a routine boot silently apply whatever migrations a new binary happens to embed. The operator never ran a dry-run, never saw the pending list, never consented to the irreversible ones.

| Question | Where the answer lives today |
| --- | --- |
| Who applies versioned schema at boot? | `bootstrap_for_serving` — gated, but the gate is open by default in dev (`Makefile:401,690,709`) |
| Who applies versioned schema in prod (documented)? | `edgequake migrate` CLI only ([runbook step 2–5](../../docs/operations/spec091-upgrade-from-v0.22.0.md)) |
| Can the operator preview impact before apply? | Yes — `edgequake migrate dry-run` exists ([15 §7.0](15-migration-console-cli.md)) but nothing forces its use |
| What stops a stale replica from mutating schema? | Nothing, when the flag is set; the fail-closed gate when unset (`migration_bootstrap/mod.rs:830-835`) |
| What DDL still runs at boot regardless? | Additive object-ensure: vector `create_table`, AGE graph lifecycle, audit partition (§6) |

**Five WHYs.** (1) Why can a boot apply irreversible migrations without consent? Because `EDGEQUAKE_ALLOW_BOOT_MIGRATE=1` folds schema apply into server start. (2) Why does that flag exist and default on in dev? To make cold-start `make dev` "just work" on a fresh or behind database. (3) Why is that the wrong trade? It optimizes the first five minutes at the cost of making the most dangerous operation the least visible one. (4) Why does that matter beyond dev? Flags leak: the same env var ends up in shell exports, start scripts, and container images — the dev escape becomes a production accident. (5) Why not just keep the flag for prod emergencies? Two writers with a toggle is one writer plus an accident; the CLI already *is* the emergency path — one command, with a dry-run.

**Causal chain:**

```ascii
 schema apply folded into boot (flag on)
   → deploy/restart/scale-out silently mutates versioned schema
     → no dry-run, no pending-list review, no consent for irreversibles
       → unplanned lock/rewrite/drop mid-roll
         → mixed-schema fleet, 42P01 on stale replicas, restore-from-backup rollback
```

**Axiom (the whole design derives from this):** *Applying versioned schema is exactly one explicit, operator-invoked, previewable action. Server start is never that action.* Boot may **verify** the schema and **refuse** when it disagrees with the binary — fail-closed — but the only path that mutates `_sqlx_migrations` is `edgequake migrate`, and the only way to reach it informed is `edgequake migrate dry-run`. This is LAW-D5 ("one schema owner") and LD-03 ("runtime code never issues DDL") completed: today they govern *which relations* migrations own; this doc governs *which process* may run them.

---

## 2. First principles → design axioms (`LAW-B1..B5`)

| Law | Statement | Anchored to |
| --- | --- | --- |
| **LAW-B1 — One schema writer: the operator-invoked CLI** | `edgequake migrate` is the only process that applies versioned migrations. No serving binary, dev script, or container entrypoint may apply them as a side effect of starting. | LAW-D5, LAW-D6 (one writer / SSOT); `run_postgres_migrations` (`migration_bootstrap/mod.rs:840`) |
| **LAW-B2 — Boot verifies and refuses, fail-closed** | At server start the bootstrap reads `_sqlx_migrations`, compares against the embedded migrator, and on any disagreement (pending **or** database-newer-than-binary) refuses to serve with a non-zero exit and an actionable message. Unknown/unreadable state ⇒ refuse. | LAW-C4 (fail-closed); `bootstrap_for_serving` (`migration_bootstrap/mod.rs:822-837`) |
| **LAW-B3 — Every refusal teaches the preview** | Each refusal message names the pending count and the two commands — `edgequake migrate dry-run` (preview) and `edgequake migrate` (apply) — plus the runbook path. An operator who hits the gate can reach the preview in one step. The message is a single builder, contract-tested for drift. | LD-14 (schema-derived guidance); `migrate_console::print_apply_intent` |
| **LAW-B4 — Additive idempotent object-ensure is not a migration** | Serving boot may still *ensure* additive, idempotent, non-versioned objects its traffic requires (per-workspace vector tables, graph labels, next audit partition) — `CREATE … IF NOT EXISTS` only, never `DROP`/rewrite of versioned relations. This class is enumerated (§6), bounded, and may never grow a destructive member. | LD-03 partial; `vector/ddl.rs:206-216`, `migration_bootstrap/mod.rs:1332` |
| **LAW-B5 — Downgrade is refused as loudly as upgrade** | A binary booting against a database whose applied version exceeds the binary's embedded latest refuses with a distinct "database newer than binary" message. Silent downgrade-serve is schema drift by omission. | `_sqlx_migrations` vs `MIGRATOR.migrations` (embedded, `migration_bootstrap/mod.rs:246`) |

These extend the existing law set; they do not amend LAW-D/C/Q/P. Registry: [04-cross-ref-matrix.md](04-cross-ref-matrix.md) § ID namespaces.

---

## 3. Current state — verified facts (code is law)

| # | Fact | Code locus |
| --- | --- | --- |
| 1 | Serving boot fails closed on pending migrations **only when** `EDGEQUAKE_ALLOW_BOOT_MIGRATE` unset and not CLI mode | `edgequake/crates/edgequake-api/src/state/migration_bootstrap/mod.rs:822-835` |
| 2 | The dev path opens that gate by default | `Makefile:690`, `Makefile:709` (`make dev`); `Makefile:401` (`backend-bg` start script) |
| 3 | The CLI path bypasses the gate via `EDGEQUAKE_MIGRATE_CLI=1` set in-process | `edgequake/src/main.rs:662`; read at `migration_bootstrap/mod.rs:811-819` |
| 4 | `edgequake migrate` already: lists pending, labels irreversibles, refuses **125**/**126** without `--confirm-drop`, offers `dry-run` | `edgequake/src/main.rs:660-785`; `edgequake/src/migrate_console.rs` |
| 5 | Support-reconcile hooks degrade to read-only catalog probes on a gated serving boot | `execute_bootstrap_apply_sql` (`state/migration_bootstrap/reconcile/mod.rs:15-28`) |
| 6 | Ungated additive DDL at every boot: vector `create_table` + ANN index, AGE graph + eq_* columns/triggers, next audit partition, dimension-driven DROP/recreate of empty vector tables | `vector/ddl.rs:193+`; `graph_lifecycle.rs`; `migration_bootstrap/mod.rs:1332`; `vector/migration.rs:115-162` |
| 7 | Published v0.22.0 image auto-migrates at boot (predates the gate) | pin `36c45b7`; `docker-compose.spec091-soak.yml:46` sets the flag for it |
| 8 | Tests never rely on boot for schema — they provision scratch DBs with the embedded migrator | `tests/common/test_db.rs:33-139`; `postgres_test_config.rs:157-192` |
| 9 | `/health` already reports `schema.latest_version` + `migrations_applied` | `services/health_schema.rs` (`fetch_sqlx_migration_stats`); `handlers/health.rs` |
| 10 | Upgrade runbook already orders migrate before server start; soak boots HEAD with the flag off | `docs/operations/spec091-upgrade-from-v0.22.0.md:50-58,111`; `scripts/spec091_upgrade_soak.sh:178` |
| 11 | No explicit `pg_advisory_lock` in code; concurrent apply is serialized by sqlx's built-in migration lock | sqlx `MIGRATOR.run()` (`migration_bootstrap/mod.rs:917`) |
| 12 | There is **no** downgrade (db-newer-than-binary) detection today | — (gap; LAW-B5) |

---

## 4. The contract

### 4.1 Boot decision (single gate, DRY)

```ascii
 server boot
   └─ bootstrap_for_serving(admin_pool)
        ├─ read _sqlx_migrations                      (verify)
        ├─ pending = embedded \ applied
        ├─ pending ≠ ∅        → REFUSE (exit 78, msg §4.2)     LAW-B2
        ├─ max(applied) > max(embedded)
        │                     → REFUSE (exit 78, downgrade msg) LAW-B5
        └─ up to date         → serve; reconcile stays read-only probes
```

One function owns this decision (`bootstrap_for_serving`); the serving path has no second gate. `EDGEQUAKE_MIGRATE_CLI=1` (set in-process by the CLI, `main.rs:662`) remains the sole bypass — it *is* the operator-invoked writer (LAW-B1).

### 4.2 Refusal message (single builder, contract-pinned — LAW-B3)

```text
error: database schema is behind this binary: N pending migration(s): [v1, v2, …].

Schema changes are explicit and previewable. Next steps:
  1. edgequake migrate dry-run     # preview the pending migrations (zero writes)
  2. edgequake migrate             # apply them (irreversible steps still require --confirm-drop)

Runbook: docs/operations/spec091-upgrade-from-v0.22.0.md
```

Downgrade variant (LAW-B5): `error: database is NEWER than this binary (applied vA > embedded vE). Run the binary that matches the schema, or restore a compatible backup.`

### 4.3 Exit code

**78** (`EX_CONFIG`) for both refusal classes — distinct from generic startup failure (1) so orchestrators and scripts can branch on "migrate required" vs "crash".

### 4.4 Health surfacing

`/health.schema` gains `pending_count: int` and `migration_required: bool` (derived from `_sqlx_migrations` vs embedded, same derivation as the gate — LAW-B3 reuse, no second computation). A boot that refused never reaches `/health`; these fields serve post-boot drift detection (e.g. a replica that stayed up while the fleet migrated) and K8s/exec gating.

### 4.5 Removed flag

`EDGEQUAKE_ALLOW_BOOT_MIGRATE` is removed as a behavior input. If still present in the environment, boot emits **one** WARN ("removed in v0.23; ignored — schema apply is `edgequake migrate` only") and proceeds to the fail-closed gate. One release of warn-and-ignore, then the WARN is dropped (changelog-tracked).

---

## 5. The process per environment

| Environment | Schema owner | Boot behavior | Change vs today |
| --- | --- | --- | --- |
| **Dev (`make dev`/`dev-bg`/`backend-bg`)** | `make` runs `cargo run -p edgequake -- migrate` as a **visible step** after `db-wait`, before server start; its preflight/guard output streams to the terminal; failure aborts the target | Verify-only; refuses if a migration appeared between the migrate step and boot (new checkout mid-run) | Flag defaults deleted (`Makefile:401,690,709`); migrate step added |
| **Production** | Operator runs `edgequake migrate dry-run` → `migrate [--confirm-drop]` per runbook | Verify-only (unchanged from documented path) | None — runbook already compliant; §4 message now standardizes the refusal |
| **Docker / Compose** | Optional one-shot `migrate` service (same image, `command: ["migrate"]`, `depends_on` healthy postgres) documented in runbook; app services stay flag-free | Verify-only | Docs only; compose files unchanged |
| **Kubernetes** | `Job` (or helm pre-upgrade hook) running `edgequake migrate [--confirm-drop]` before the Deployment rolls; readiness can gate on `/health.schema.migration_required == false` | Verify-only; CrashLoopBackOff until the Job completes — visible, not silent | Docs/example manifest |
| **Tests** | Test harness provisions scratch DBs with the embedded migrator (`test_db.rs`, `postgres_test_config.rs`) — unchanged | In-process boots find zero pending → pass the gate with no flag | None; contract test pins this |
| **CI** | `migration-guard.yml` uses `sqlx migrate run` on scratch DBs — unchanged | n/a | None |
| **Old v0.22.0 image** | Pre-gate behavior (auto-migrates); upgrade runbook already rolls write-stop binary first, then migrates via CLI | n/a (pin) | Documented as historical note only |

**First-boot (fresh install) UX:** server refuses with the §4.2 message listing all pending migrations; operator runs `dry-run` then `migrate`. `make dev` performs both visibly. This is the intended friction: the first schema is the one schema change every install must consent to.

**Rolling upgrade (multi-replica):** roll replicas to the new binary → they refuse while pending exist (visible crash-loop) → operator previews and applies once → replicas start on next restart. This is the same ordering the SPEC-091 runbook already mandates (write-stop fleet, then migrate), now enforced by the binary instead of by discipline.

---

## 6. Additive object-ensure vs versioned migrations (LAW-B4 classification)

Boot-time DDL falls into exactly two classes. **Class V (versioned migrations)** is `_sqlx_migrations`-tracked, ordered, potentially irreversible — CLI-only after this change. **Class A (additive object-ensure)** is idempotent `CREATE … IF NOT EXISTS` for objects serving traffic creates dynamically — it stays at boot, is enumerated here, and may never add a destructive member without a new locked decision.

| Object | Class | Locus | Notes |
| --- | --- | --- | --- |
| All `NNN_*.sql` migrations + support reconcile apply | **V** | `migrations/`; `run_postgres_migrations` | CLI-only (LAW-B1) |
| Per-workspace vector tables + ANN index (`CREATE TABLE/INDEX IF NOT EXISTS`) | A | `vector/ddl.rs:206-241` | Serving-created per (namespace, workspace); additive |
| AGE graph + eq_* columns/triggers ensure | A | `graph_lifecycle.rs` | Idempotent ensure; additive |
| Next audit-log partition ensure | A | `migration_bootstrap/mod.rs:1332` | Time-derived, additive |
| `eq_hot_ann_workspaces` registry ensure | A | `vector/ddl.rs:585+` | Additive |
| **Dimension-driven DROP/recreate of empty vector tables** | **V-leaning exception** | `vector/migration.rs:115-162` | Drops only **empty** tables, but `DROP` is destructive-shaped. Stays at boot for this phase (SPEC-058 behavior, `PreferExisting` default); flagged for a future gate — new locked decision required to keep or retire. Documented here so it cannot grow silently. |
| Serving-fence state, migration-engine job rows | A (data, not schema) | migrations 106/109 + engine | Data-plane, governed by LD-08, orthogonal |

---

## 7. Waves

### B0 — Spec & registry (this doc)

- **Entry:** decision locked (refuse-by-default; explicit migrate via make).
- **Mechanism:** doc 17 + LD-15 in [README.md](README.md); LAW-B/EC-B/R-B namespaces in [04-cross-ref-matrix.md](04-cross-ref-matrix.md); planned tests in [11-e2e-test-matrix.md](11-e2e-test-matrix.md).
- **Exit gate:** registry consistent; no code changed.
- **Rollback:** delete doc + registry lines (docs only).

### B1 — Gate hardening (code)

- **Entry:** B0 merged.
- **Mechanism:** remove `allow_boot_migrate()` + flag branch (`migration_bootstrap/mod.rs:799-808,830-835,859-866`); single refusal-message builder (§4.2) reused by both refusal sites; exit 78; downgrade detection (LAW-B5); stale-flag WARN; `/health.schema.pending_count` + `migration_required`.
- **Flag:** none — behavior change is the feature; stale env var warns for one release (§4.5).
- **Exit gate:** `contract_spec091_boot_gate` green (§10); `cli_migrate_console` still green; clippy `-D warnings`, fmt clean.
- **Rollback:** revert commit; flag branch restored (pure code, no schema state).

### B2 — Environment alignment (make, docker, docs)

- **Entry:** B1 merged.
- **Mechanism:** Makefile dev targets drop flag defaults + add visible migrate step; compose/K8s one-shot examples in runbook; `.env.example` cleanup; runbook + release-and-cd + CHANGELOG updates.
- **Exit gate:** fresh DB → `make dev` shows migrate preflight, applies, then boots healthy; behind DB → server refuses with §4.2 message; soak `make spec091-upgrade-soak` still green.
- **Rollback:** revert Makefile/docs (no code).

**Sequencing invariant:** B1 must land before or with B2's Makefile change — removing the dev flag without the hardened refusal message would leave dev cold-start with a bare error.

---

## 8. Edge cases (`EC-B1..B14`)

| EC | Scenario | Behavior (per contract) | Owning wave | Test |
| --- | --- | --- | --- | --- |
| EC-B1 | Fresh empty DB, server boot | Refuse exit 78; message lists all pending; `dry-run`+`migrate` named | B1 | `contract_spec091_boot_gate::fresh_db_refuses` |
| EC-B2 | Schema behind (N pending), server boot | Refuse exit 78 with exact pending version list | B1 | `contract_spec091_boot_gate::behind_db_refuses_lists_pending` |
| EC-B3 | Schema newer than binary (downgrade) | Refuse exit 78, distinct downgrade message; no silent serve | B1 | `contract_spec091_boot_gate::newer_db_refuses` |
| EC-B4 | Multi-replica rolling deploy before migrate | All new replicas refuse (crash-loop, visible); operator migrates once; replicas start | B2 (docs) | runbook assertion in soak |
| EC-B5 | `migrate` CLI applying while replicas boot | sqlx advisory lock serializes apply; replicas see pending → refuse; restart after apply completes | B1 | covered by CLI tests + soak tee |
| EC-B6 | Crash-loop orchestrator (K8s/systemd) | Restart until migrated — visible failure, not silent drift; Job/hook pattern + `/health` gate documented | B2 | docs example; `/health` field test |
| EC-B7 | `make dev` cold start on fresh/behind DB | Visible migrate step streams preflight; failure aborts target before server starts | B2 | `make dev` smoke (manual + script) |
| EC-B8 | Test harness (scratch DBs) | Harness pre-migrates; in-process boot sees zero pending → passes with no flag | B1 | existing suites stay green (no flag in CI) |
| EC-B9 | `EDGEQUAKE_ALLOW_BOOT_MIGRATE` still set in env | One WARN ("removed, ignored"); fail-closed gate applies; no silent apply | B1 | `contract_spec091_boot_gate::stale_flag_warns_and_does_not_apply` |
| EC-B10 | Boot with additive object-ensure needed (new workspace vector table) | Ensured at boot (Class A) — not a migration; refuses nothing | B1 (classification) | existing workspace e2e stay green |
| EC-B11 | Dimension mismatch on empty vector table at boot | DROP/recreate runs (documented exception, SPEC-058); logged; future gate TBD — must not grow silently | spec-only | existing dimension e2e stay green |
| EC-B12 | `dry-run` with irreversibles pending | Already labels **125**/**126** IRREVERSIBLE + RISK block — unchanged, referenced by refusal message | B1 | `cli_migrate_console` (existing) |
| EC-B13 | `EDGEQUAKE_MIGRATION_MODE=automatic` at boot | Engine data backfills resume (LD-08, non-blocking) — orthogonal to schema gating; unchanged | out of scope | engine e2e stay green |
| EC-B14 | Old v0.22.0 image boot | Pre-gate auto-migrate (pin behavior); runbook rolls write-stop binary first — documented, not changed | B2 (docs) | soak continues to prove path |

## 9. Risks (`R-B1..B5`)

| ID | Risk | L | I | Mitigation | Early warning | Owner |
| --- | --- | --- | --- | --- | --- | --- |
| R-B1 | Dev friction: cold-start `make dev` slower/fails visibly where it once silently fixed itself | M | L | Visible migrate step with streamed preflight; failure message names dry-run; one command to fix | dev complaints; support pings | B2 |
| R-B2 | Orchestration crash-loops read as "outage" by teams used to auto-migrate | M | M | Runbook K8s Job/hook pattern; `/health` gating fields; CHANGELOG behavior-change banner | alert noise on first deploy post-release | B2 |
| R-B3 | Flag removal breaks external automation relying on `EDGEQUAKE_ALLOW_BOOT_MIGRATE` | L | M | One-release warn-and-ignore; CHANGELOG + release-note callout; flag was never in `.env.example` | WARN sightings in logs | B1 |
| R-B4 | Refusal message drifts from actual commands (dry-run renamed, runbook moved) | L | M | Single message builder + contract test pinning the three elements (count, both commands, runbook path) — LAW-B3 | contract test failure | B1 |
| R-B5 | Operator runs `migrate` against the wrong database | L | H | `dry-run`/preflight print redacted URL banner (existing); refusal message points to dry-run first | preflight URL mismatch reviews | — (existing mitigation) |

## 10. Test matrix additions

| Test | Kind | Wave | Asserts |
| --- | --- | --- | --- |
| `contract_spec091_boot_gate::fresh_db_refuses` | contract | B1 | EC-B1: exit 78, message has count + `dry-run` + `migrate` + runbook |
| `contract_spec091_boot_gate::behind_db_refuses_lists_pending` | contract | B1 | EC-B2: exact pending versions in message |
| `contract_spec091_boot_gate::newer_db_refuses` | contract | B1 | EC-B3: downgrade message, exit 78 |
| `contract_spec091_boot_gate::boot_succeeds_after_cli_migrate` | contract | B1 | same scratch DB: refuse → `run_postgres_migrations` (CLI mode) → boot gate passes |
| `contract_spec091_boot_gate::stale_flag_warns_and_does_not_apply` | contract | B1 | EC-B9: flag set → gate still refuses, schema untouched |
| `contract_spec091_boot_gate::health_reports_pending` | contract | B1 | `/health` derivation: `pending_count`, `migration_required` (unit-level on the derivation fn) |
| `cli_migrate_console` (existing) | e2e CLI | — | must stay green (EC-B12 guard) |
| `make dev` smoke (documented manual/script) | ops | B2 | EC-B7 both branches (fresh → applies+boots; behind → visible step) |

Register under [11-e2e-test-matrix.md](11-e2e-test-matrix.md) with an honest "Planned" → "Exists today" flip on B1 merge.

## 11. Acceptance criteria

- [ ] Doc 17 + LD-15 + namespace registry merged (B0).
- [ ] No code path outside `edgequake migrate` applies versioned migrations; `EDGEQUAKE_ALLOW_BOOT_MIGRATE` no longer read (WARN-only) (B1).
- [ ] Boot refusal: exit 78, message contains pending count, `edgequake migrate dry-run`, `edgequake migrate`, runbook path — contract-pinned (B1).
- [ ] Downgrade (db newer than binary) refuses with distinct message (B1).
- [ ] `/health.schema.pending_count` + `migration_required` present and correct (B1).
- [ ] `make dev` performs a visible migrate step; flag defaults removed from all targets (B2).
- [ ] Runbook documents Docker one-shot + K8s Job patterns; CHANGELOG records the behavior change (B2).
- [ ] `cargo test -p edgequake-api --features postgres --test contract_spec091_boot_gate` green; `cli_migrate_console` green; clippy `-D warnings`; fmt clean; `make spec091-upgrade-soak` green.

## 12. Related

- Console/CLI (the sole schema writer): [15-migration-console-cli.md](15-migration-console-cli.md)
- Engine + upgrade path: [07-migration-engine.md](07-migration-engine.md)
- LD registry: [README.md § Locked decisions](README.md#locked-decisions) (LD-15)
- ID registry: [04-cross-ref-matrix.md](04-cross-ref-matrix.md)
- Operator runbook: [docs/operations/spec091-upgrade-from-v0.22.0.md](../../docs/operations/spec091-upgrade-from-v0.22.0.md)
- HEAD data-model truth: [16-post-cutover-assessment.md](16-post-cutover-assessment.md)
