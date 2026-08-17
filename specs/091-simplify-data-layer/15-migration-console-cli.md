# 15 — Migration Console (CLI): Embedding the Migration Intelligence

> **Status:** IMPLEMENTED (C0–C3 in working tree) — `edgequake migrate status|console|plan|guard|family|pause|resume|cancel|dry-run` plus gated `--confirm-drop` for migration 125. **Wired CI:** `e2e_spec091_console`, `cli_migrate_console` ([11 § Exists today](11-e2e-test-matrix.md#exists-today-run-these)). Aspirational names in §10–11 (`e2e_spec091_console_family`, `contract_spec091_advisor_soak_gate`, …) are design targets, not current binaries. WebUI/admin-API advisor surface remains out of scope.
> **Scope:** CLI console (`edgequake migrate …`). The Advisor is surface-agnostic so a future WebUI/admin-API console can consume the same SSOT without re-derivation.
> **Builds on:** [07-migration-engine.md](07-migration-engine.md) (ledger, state machine, four surfaces) · [05-target-specification.md](05-target-specification.md) · [06-implementation-plan.md](06-implementation-plan.md)
> **Laws:** LAW-D5 (no runtime DDL), LAW-D6 (one writer / SSOT), LAW-D8 (data movement is a descriptor, never boot-blocking), LD-07 (flag-gated change, one irreversible op per release), LD-08 (monotonic progress)

---

## 1. WHY — the operator's brain is the (missing) integration layer

The migration **already knows everything**, but the knowledge is scattered across raw surfaces and the operator must integrate it by hand. Today, to answer *"where am I in the KV retirement, and what is the next safe action?"* an operator must manually correlate:

| Question                             | Where the answer lives today                                   |
| --------------------------------------| ----------------------------------------------------------------|
| Is the chunk backfill done?          | `edgequake migrate status` → raw `state`/`completion_pct`      |
| Did verification pass?               | the job's `VerifyReport` (not surfaced)                        |
| Which families are still writing KV? | `EDGEQUAKE_KV_FAMILY_*` + `EDGEQUAKE_CHUNK_TEXT_AUTHORITY` env |
| Is the KV store safe to drop?        | the durable-row guard inside `125_spec091_kv_drop.sql`         |
| What do I do **next**?               | **nothing computes this** — read the spec docs and guess       |

**Five WHYs.** (1) Why did the post-drop `dual` bug happen? A stale flag wrote to a dropped table. (2) Why was the flag stale? Nothing surfaced that the drop had already run. (3) Why not? The drop state lives in `_sqlx_migrations`, the flag in the environment — no single view joins them. (4) Why no single view? The surfaces report *raw* state, never *derived* posture. (5) Why? The console was built to **show** the migration, not to **guide** it.

**Causal chain:**

```ascii
 intelligence scattered + raw
   → operator integrates by hand
     → stale flag / wrong sequence / premature drop
       → 42P01, data-loss risk, rollback panic
```

**Axiom (the whole design derives from this):** *The database schema is already the single source of truth. The console's job is to **derive** and **present** that truth — never to persist a parallel copy (SSOT) — and to **refuse** any illegal or unsafe transition (gated guardrails).* Guidance that lives only in the spec (or in someone's head) is a bug waiting to be re-introduced; guidance derived live from the schema cannot go stale.

---

## 2. First principles → design axioms (`LAW-C1..C6`)

| Law | Statement | Anchored to |
| --- | --- | --- |
| **LAW-C1 — Derive, never store** | The console persists nothing about migration posture; every number is computed from the schema on each invocation. | SSOT axiom; `migration_progress` view (`migrations/106`) |
| **LAW-C2 — One advisor, many renderers** | A single `migration_advisor` module produces the posture; the CLI is a thin renderer. No second place may compute "what next" (DRY). | DRY; `readiness_blockers` SSOT pattern (`state/migration_bootstrap/mod.rs:682`) |
| **LAW-C3 — Reuse the real guard, verbatim** | Readiness/flip/drop checks call the *same* SQL/functions the runtime and migration-125 guard use — never a re-implementation that can diverge. | `125_spec091_kv_drop.sql:39-109`; `verify.rs` |
| **LAW-C4 — Fail-closed** | Unknown state, unreadable ledger, or an ambiguous signal ⇒ the action is refused and the posture is reported as "cannot determine", never guessed. | 07 §Gates "fail-closed"; `kv.rs:116-118` 42P01 tolerance |
| **LAW-C5 — Irreversible is explicit, recorded, one-per-release** | Destructive steps are never *executed* by the console; they are gated behind a recorded `--confirm-*` token and stay in migration-owned SQL. | LD-07; `125_spec091_kv_drop.sql` (IRREVERSIBLE) |
| **LAW-C6 — Progress is a number, guidance is a sentence** | Alongside every metric the console emits the explicit operator instruction that metric implies. | LD-08; 07 §Progressive information |

---

## 3. The intelligence model — what the schema already knows

Every fact the console needs, its **current schema source**, and **how the advisor derives it**. This is the grounding contract: each row is a real, queryable source today.

| # | Fact | Schema source (code is law) | Derivation |
| --- | --- | --- | --- |
| 1 | Job state / progress / ETA | `edgequake.edgequake_migration_job` + `..._batch` + `edgequake.migration_progress` view (`migrations/106_spec091_migration_engine.sql`) | `lease::job_detail` → state, `completion_pct`, `rows_per_sec`, `eta_seconds`, `throttle_reason`, `last_error` |
| 2 | Is the KV store dropped? | `_sqlx_migrations` (version 125 applied) + `pg_class` for `public.eq_%_kv` | `kv_store_dropped: bool` = (125 applied) OR (no `eq_%_kv` relations remain) |
| 3 | Per-family write/read mode | `EDGEQUAKE_KV_FAMILY_<FAMILY>` (`kv_family_cutover.rs:34`) + `EDGEQUAKE_CHUNK_TEXT_AUTHORITY` (`chunk_text_authority.rs`) | `Kv` \| `Dual` (chunk only) \| `Relational` (default since Wave D) |
| 4 | Verification result | job's `VerifyReport` (`runner.rs:36-48`, `passes() = mismatches==0 && actual>=expected`) | re-run `verify::verify_chunk_text_backfill` read-only, or read recorded result |
| 5 | KV residue per family (drop-readiness) | the durable-row guard SQL (`125_spec091_kv_drop.sql:57-101`) | **reuse verbatim**: per-family durable-row count not yet in its typed SSOT |
| 6 | Typed SSOT row counts | `public.chunks`, `public.documents`, `public.llm_cache`, `ingestion_dedup`, `document_artifacts`, `pipeline_checkpoints`, `compensation_quarantine` | `COUNT(*)` per typed table |
| 7 | Serving-fence posture | `EDGEQUAKE_SERVING_FENCE` + `edgequake.chunk_serving_state` (`serving_fence.rs`, migration 109) | fence on/off + per-state chunk distribution |
| 8 | Migration engine mode | `EDGEQUAKE_MIGRATION_MODE` (`migration_engine/mod.rs`: off/verify/automatic) | whether the engine will execute or only report |

The eight facts above are the **entire** input set. The advisor needs nothing else — this is what "based on the current schema of the database" means concretely.

---

## 4. The Advisor — one derived posture (SSOT, DRY)

A single module `edgequake_storage::migration_engine::advisor` (feature `postgres`) exposes:

```rust
/// Derived, never persisted (LAW-C1). Recomputed from the schema per call.
pub struct MigrationPosture {
    pub kv_store_dropped: bool,          // fact #2
    pub engine_mode: EngineMode,         // fact #8 (off | verify | automatic)
    pub serving_fence_on: bool,          // fact #7
    pub families: Vec<FamilyPosture>,    // per-family derived phase
    pub cutover_phase: CutoverPhase,     // aggregate (bottleneck family wins)
    pub next_instructions: Vec<Instruction>,   // §5 — explicit, ordered
    pub available_actions: Vec<GuardedAction>, // §6 — gated
}

pub struct FamilyPosture {
    pub family: &'static str,            // CHUNK | METADATA | ... | CACHE
    pub mode: FamilyMode,                // Kv | Dual | Relational (fact #3)
    pub phase: FamilyPhase,              // state machine below
    pub backfill: Option<JobSnapshot>,   // state, pct, eta (fact #1)
    pub verify: Option<VerifyReport>,    // fact #4
    pub kv_residue_rows: Option<i64>,    // fact #5 (None once dropped)
    pub typed_rows: i64,                 // fact #6
}

pub enum FamilyPhase {
    NotStarted,     // mode=Kv, no typed migration yet for this family
    DualWriting,    // chunk only: mode=Dual, backfill incomplete
    Backfilling,    // job running/paused, pct < 100
    Verifying,      // job state = verifying
    ReadyToFlip,    // job completed ∧ verify passes ∧ residue typed-verified, mode still Kv/Dual
    Flipped,        // mode=Relational, KV relations still present (soaking)
    ReadyToDrop,    // all families Flipped ∧ total durable residue = 0
    Dropped,        // KV relations gone
}
```

**Per-family phase state machine (ASCII):**

```ascii
            mode=Kv                                   backfill job
            (no typed migration)                       running
 ┌────────────┐   flag dual    ┌─────────────┐  pct<100  ┌─────────────┐
 │ NotStarted │──────────────▶│ DualWriting │──────────▶│ Backfilling │◀─┐
 └────────────┘  (chunk only) └─────────────┘           └──────┬──────┘  │
                                                                │ paused │
                                                                ▼        │
                                                        ┌─────────────┐  │
                                                        │  Verifying  │──┘
                                                        └──────┬──────┘
                                           verify passes ∧ residue typed-verified
                                                                ▼
   ┌────────────┐   all Flipped ∧ residue=0   ┌─────────┐   ┌─────────────┐
   │ ReadyToDrop│◀────────────────────────────│ Flipped │◀──│ ReadyToFlip │
   └─────┬──────┘                             └─────────┘   └─────────────┘
         │ migration 125 applied (IRREVERSIBLE, operator-confirmed)
         ▼
   ┌──────────┐
   │ Dropped  │   ← terminal; chunk authority = public.chunks (SSOT)
   └──────────┘
```

`CutoverPhase` (the single "where am I") is the aggregate: the **earliest-phase durable family** determines the global phase (a cutover is only as advanced as its slowest durable family).

**Reuse, not re-implementation (LAW-C3).** The advisor calls the *same* building blocks the runtime uses — `migration_progress` view, `lease::job_detail`, `verify::verify_chunk_text_backfill`, and the migration-125 durable-row SQL — exposed behind a `PostureSource` port so the rule engine is a **pure function** over the posture (unit-testable without a database). A **contract test asserts the advisor's drop-readiness verdict always equals the migration-125 guard's verdict** on the same database, so the two can never silently diverge.

```mermaid
flowchart TD
    subgraph SSOT["Database schema (SSOT — never duplicated)"]
        L["migration_job/_batch + migration_progress view"]
        V["_sqlx_migrations (125 = dropped)"]
        R["eq_*_kv residue (migration-125 durable-row SQL)"]
        T["typed tables: chunks/documents/llm_cache/..."]
        S["chunk_serving_state (fence)"]
    end
    ENV["EDGEQUAKE_KV_FAMILY_* / CHUNK_TEXT_AUTHORITY / MIGRATION_MODE / SERVING_FENCE"] --> ADV
    SSOT --> ADV["advisor: PostureSource port → Postgres adapter"]
    ADV --> RULES["pure rule engine: posture → instructions + guarded actions"]
    RULES --> CLI["edgequake migrate console / plan / family / guard / pause|resume|cancel"]
    CLI -. "gated writes only (pause/resume/cancel/family set)" .-> L
```

---

## 5. Explicit instructions — the decision table

Guidance is **data**, not prose. The rule engine maps derived posture → an ordered `Vec<Instruction>`:

```rust
pub struct Instruction {
    pub ordinal: u32,             // display order (runbook sequence)
    pub kind: InstrKind,          // Wait | Action | Confirm | Done | Blocked
    pub summary: String,          // one line, present tense
    pub command: Option<String>,  // exact flag/command to run (copy-pasteable)
    pub gate: Option<String>,     // the gate that must be green first
    pub evidence: String,         // the numbers behind it (pct, mismatches, residue)
}
```

**Decision table (excerpt — the full table is the rule engine's unit-test surface):**

| Derived state (posture) | Kind | Instruction |
| --- | --- | --- |
| engine_mode=off | Blocked | "Engine is `off` — jobs register but never run. Set `EDGEQUAKE_MIGRATION_MODE=automatic` to execute, or stay `verify` to only observe." |
| chunk Backfilling, pct<100 | Wait | "WAIT: chunk backfill {pct}% (ETA {eta}). Do **not** flip flags yet." |
| chunk job=completed ∧ verify 0 mismatches ∧ mode=dual | Action | "ACTION: `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=relational`, restart. Verified {sampled} samples, {mismatches} mismatches." |
| family F ReadyToFlip | Action | "ACTION: `EDGEQUAKE_KV_FAMILY_{F}=relational`, restart. Residue typed-verified ({residue} rows confirmed in {typed_table})." |
| any mode=kv/dual ∧ kv_store_dropped | Blocked | "BLOCKED: `{flag}={mode}` but the KV store is **dropped** — this writes to a missing relation (42P01). Set `{flag}=relational` immediately." **(this is the exact stale-flag bug, now caught)** |
| all Flipped ∧ soak incomplete | Wait | "WAIT: soak in progress ({elapsed}/{window}). LD-07 requires a full soak before the drop." |
| all Flipped ∧ residue=0 ∧ KV present ∧ soak done | Confirm | "CONFIRM: KV store drained + verified. Apply migration 125 (IRREVERSIBLE): `edgequake migrate` with `--confirm-drop`. Rollback after = restore from backup." |
| residue>0 on drop check | Blocked | "BLOCKED: {residue} durable KV rows not yet in typed tables ({families}). Run the family backfills (117-122/engine) and re-check." |
| kv_store_dropped ∧ all relational | Done | "DONE: generic KV store retired. Chunk text authority = `public.chunks`; all families relational." |

---

## 6. Gated guardrails — the control plane

Every mutation the console offers is a `GuardedAction`, re-checked against the schema **at execution time** (never trust a stale posture) and refused with an explicit reason when gated:

```rust
pub struct GuardedAction {
    pub verb: String,                 // "pause" | "resume" | "cancel" | "family.set" | "drop"
    pub target: String,               // step_id or family
    pub enabled: bool,
    pub gate_reason: Option<String>,  // why disabled (operator-readable)
    pub requires_confirmation: bool,  // irreversible / destructive
    pub irreversible: bool,
}
```

**Guardrail matrix:**

| Action | Gate (checked live) | Refuse reason when gated |
| --- | --- | --- |
| `pause <step>` | `lease::control_job` transition legal from current state (`lease.rs:186-196`) | "cannot pause: job is `{state}` (legal from pending/preflight/running/verifying)" |
| `resume <step>` | state = paused | "cannot resume: job is `{state}`, not paused" |
| `cancel <step>` | state ∈ pending/preflight/running/paused | "cannot cancel: job already `{state}`" |
| `family set <F> relational` | F = ReadyToFlip (job completed ∧ verify passes ∧ residue typed-verified) | "cannot flip {F}: {reason} (backfill incomplete / {mismatches} verify mismatches / {residue} un-migrated rows)" |
| `family set <F> kv` (rollback) | KV relations still exist | "cannot roll back {F} to `kv`: KV store dropped — would 42P01 (the bug)" |
| `family set CHUNK dual` | KV relations still exist | "cannot dual-write: KV store dropped" |
| `drop` | **never executed by CLI**; read-only guard report only | n/a — the drop runs only via migration 125 under `--confirm-drop` |

Cross-cutting: **fail-closed** (LAW-C4) — any unreadable ledger / ambiguous signal ⇒ `enabled=false` with `gate_reason="cannot determine state — refusing"`. **Idempotent** — re-running an already-satisfied action is a no-op success, not an error.

---

## 7. The CLI console surface

Extends `edgequake migrate` (`main.rs` arg parsing; `migrate_console.rs` print helpers). Read verbs + gated control verbs, including preview-only `dry-run`.

### 7.0 `edgequake migrate dry-run`

**Preview only — zero mutations.** Connects admin/query pools, prints:

1. Banner + `MODE: DRY-RUN (no changes will be applied)`
2. Pending migrations with class labels (`[expandable]` for 106–124; `[IRREVERSIBLE — KV drop]` for 125)
3. Live posture via the advisor (family table, NEXT runbook, gated actions, drop-readiness guard)
4. Operator upgrade checklist (backup, write-stop roll, relational flags, then `--confirm-drop`)

Exit **0** when the preview completed (even if drop-readiness is RED — RED is information). Exit non-zero only on connect/advisor hard errors. Must **not** call `run_postgres_migrations` or any write verb.

Full apply remains `edgequake migrate` / `edgequake migrate --confirm-drop`.

### 7.1 `edgequake migrate console [--watch]`

The intelligent dashboard. One screen: posture, next instructions, available actions.

```ascii
EdgeQuake migrate console v<pkg>
database: postgresql://edgequake:****@localhost:5432/edgequake
cutover phase: BACKFILLING (chunk)        engine: verify|automatic        serving-fence: on|off

 FAMILY      MODE        PHASE          BACKFILL                 VERIFY        RESIDUE   TYPED
 CHUNK       dual        Backfilling    42.3%  ETA 01:12:44      —             501       12,204
 METADATA    relational  Flipped        —                        —             0         3,411
 WSDOC       relational  Flipped        —                        —             0         3,411
 CACHE       kv          NotStarted     —                        —             1,204     0

 NEXT (runbook)
  1. WAIT   chunk backfill 42.3% (ETA 01:12:44) — do not flip flags yet.
  2. BLOCKED CACHE mode=kv with no typed migration — set EDGEQUAKE_KV_FAMILY_CACHE=relational
            (drained in migration 124; caches recompute on demand).

 ACTIONS  (gated)
  ✓ pause w1-chunk-text-backfill
  ✗ family set CHUNK relational   — backfill incomplete (42.3%)
  ✗ drop                          — 1,204 durable CACHE rows + 501 chunk rows not yet typed

 --watch: refreshing every 5s (Ctrl-C to exit)
```

### 7.2 `edgequake migrate plan`

The ordered, live-derived runbook (the `next_instructions` only — the "explicit instruction" deliverable in its purest form).

### 7.3 `edgequake migrate family list | set <family> <mode> [--yes]`

`list` prints the per-family posture table; `set` performs a **gated** flag change (writes the flag to the process env guidance / prints the exact export, since flags are env-owned) and refuses with `gate_reason` when the flip is not yet safe.

### 7.4 `edgequake migrate guard [--family]`

Read-only readiness probe: runs the **actual** flip-guard / drop-guard SQL and reports green/red per family with evidence. Never mutates.

### 7.5 `edgequake migrate pause|resume|cancel <step>`

Existing job-control verbs (backed by `lease::control_job`), now returning the explicit `gate_reason` on refusal. Cancel = stop after current batch, cursor retained (permanent pause, not revert — 07 §Control).

> **Design note (LAW-C5):** the CLI **never executes** the drop. `migrate guard` reports drop-readiness; the irreversible drop remains migration-125, applied by `edgequake migrate` (the schema owner) only when the guard is green **and** `--confirm-drop` is present. One irreversible op per release, recorded on the job.

---

## 8. DRY / SOLID / SSOT mapping

| Principle | How the design honors it |
| --- | --- |
| **SSOT** | The advisor persists nothing; posture is derived from `migration_progress`, `_sqlx_migrations`, env flags, typed counts, and the migration-125 guard SQL on every call (LAW-C1). |
| **DRY** | One `advisor` module; the CLI renders it. Guard logic is *reused* from `verify.rs` / `125_spec091_kv_drop.sql`, never duplicated (LAW-C3). The control verbs reuse `lease::control_job`. |
| **SOLID — SRP** | Advisor (derive) / rule engine (decide) / CLI (render) / `lease` (mutate) are four single responsibilities. |
| **SOLID — DIP/OCP** | `PostureSource` port with a Postgres adapter; the rule engine is a pure function over `MigrationPosture` — new families/instructions are added as data, not by editing control flow. |

---

## 9. Phased implementation — waves C0–C3 (each with entry/exit gates + rollback)

| Wave | Mechanism | Entry gate | Exit gate | Rollback |
| --- | --- | --- | --- | --- |
| **C0 — Advisor core (read-only)** | `advisor` module + `PostureSource` + rule engine + `migrate console`/`plan`/`guard` | 07 engine + Wave D landed; `migration_progress` view live | posture matches hand-written SQL on fixture DBs at **every** `FamilyPhase`; `contract_spec091_advisor_*` green | feature-flag the new verbs; remove |
| **C1 — Guarded job control** | `migrate pause/resume/cancel` → `lease::control_job` with gate reasons | C0 green | existing `e2e_spec091_job_control` stays green; new refusal-reason tests pass | remove verbs |
| **C2 — Family cutover control** | `migrate family list/set` gated on verify + residue | C1 green | flip refused when mismatches>0; `kv`-rollback refused post-drop; `e2e_spec091_console_family` green | remove verb |
| **C3 — Drop guard + confirmation** | `migrate guard` drop-readiness + `--confirm-drop` token plumbing into the schema-owner apply path | C2 green | advisor drop-verdict == migration-125 guard verdict (contract test) on fixture DBs incl. EC-34 residue; `e2e_spec091_wave_d` reused | remove token path |

**Flag:** `EDGEQUAKE_MIGRATION_CONSOLE=off|on` — **write verbs** (`family set`, pause/resume/cancel) are gated behind `on` (LD-07). Read-only verbs (`console`, `plan`, `guard`, `dry-run`, `family list`) are always available.

---

## 10. Edge cases (`EC-C`) & risks (`R-C`)

| ID | Edge case | Mitigation | Owning test |
| --- | --- | --- | --- |
| EC-C1 | Stale `dual`/`kv` flag **after** the drop (the real bug) | Advisor emits a `Blocked` instruction + refuses `family set … kv`; posture shows the mismatch | `e2e_spec091_console_stale_flag_post_drop` |
| EC-C2 | Fresh/empty database (no ledger, no families) | posture = NotStarted; instruction = "run `make dev` / `edgequake migrate`" | `contract_spec091_advisor_fresh_db` |
| EC-C3 | Ledger table missing (pre-106 DB) | advisor reports "engine not installed — apply migration 106" (fail-closed, not a guess) | `contract_spec091_advisor_no_ledger` |
| EC-C4 | Job paused mid-backfill | phase = Backfilling with `throttle_reason`/paused surfaced; instruction = "resume when ready" | `e2e_spec091_job_control` (reuse) |
| EC-C5 | Verify passes but residue>0 (EC-34-style KV-only rows) | family stays Backfilling/ReadyToFlip=false; drop blocked with residue count | advisor↔125-guard contract test |
| EC-C6 | Advisor run while a write is in flight | guardrails re-check live at execution; a stale displayed posture is harmless | `e2e_spec091_console_recheck_live` |
| R-C1 | Advisor diverges from the real guard over time | LAW-C3 reuse + the advisor↔125 contract test in CI | contract test (blocking) |
| R-C2 | Operator treats console output as permission to skip soak | `Flipped` requires the soak window before `ReadyToDrop`; LD-07 surfaced in the instruction text | `contract_spec091_advisor_soak_gate` |

---

## 11. Test matrix

| Test | Type | Covers |
| --- | --- | --- |
| `contract_spec091_advisor_phases` | unit (pure rule engine over fixture postures) | every `FamilyPhase` → correct instruction + gated actions |
| `contract_spec091_advisor_matches_125_guard` | integration (fixture DBs incl. EC-34 residue) | advisor drop-verdict **==** migration-125 guard verdict |
| `contract_spec091_advisor_fresh_db` / `_no_ledger` | integration | EC-C2 / EC-C3 fail-closed |
| `e2e_spec091_console_render` | e2e | `migrate console`/`plan` output shape + instruction ordering |
| `e2e_spec091_console_family` | e2e | gated `family set` (refused / allowed) |
| `e2e_spec091_console_stale_flag_post_drop` | e2e | EC-C1 (the bug) — blocked + refused |
| reuse `e2e_spec091_job_control` | e2e | C1 pause/resume/cancel legality |
| reuse `e2e_spec091_wave_d` | e2e | C3 drop guard |

---

## 12. Acceptance criteria

| Property | Criterion |
| --- | --- |
| Derivation purity | console persists zero posture rows; every number traced to a schema source (LAW-C1) |
| Guard fidelity | advisor flip/drop verdicts identical to `verify.rs` / migration-125 guard on all fixture DBs |
| Fail-closed | no action ever executes on an unreadable/ambiguous state (LAW-C4) |
| Explicit guidance | every metric row is paired with the instruction it implies (LAW-C6); `migrate plan` is always non-empty unless `Dropped` |
| Irreversibility control | the drop is never executed by the CLI; `--confirm-drop` recorded; one irreversible op per release (LD-07) |
| Stale-flag safety | a `kv`/`dual` flag against a dropped store is always surfaced `Blocked` and refused (EC-C1) |
| DRY | a second surface (future WebUI/API) can consume `advisor::posture()` with no re-derivation |
