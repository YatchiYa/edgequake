# 18 — Full-Completeness Assessment: The Six Criteria at HEAD

> **Status:** ASSESSMENT (baseline 2026-07-30) + **post-IW0–IW5 update** + **wire-closure pass**. The grades in §1 describe the *pre-program* baseline that drove [19](19-improvement-plan.md). After IW0–IW5 + wire-closure: C2 D-severity defects closed; C5 CI-wired (`make spec091-gates` + `spec091-data-layer.yml` includes IW0/IW1/IW2/IW3 contracts); C1 fleet typed (migrations 130/131) with **typed serving default**; C4 scorecard executable + CI-gated; C6 capability matrix + pg16/pg18 PR smoke; C3 phased (census allowlist + flag refuse + drain applier; facade not yet deleted). See [19](19-improvement-plan.md) status banner for residual honesty.
> **Product pin (published):** still **v0.22.0** — schema ≤ migration **105**. HEAD ships migrations **106–131** unreleased.
> **Builds on:** [16-post-cutover-assessment.md](16-post-cutover-assessment.md) · [17-boot-migration-gating.md](17-boot-migration-gating.md) · [11-e2e-test-matrix.md](11-e2e-test-matrix.md)
> **Output:** gap register `GAP-091-01..34` (§8) feeding [19-improvement-plan.md](19-improvement-plan.md).

---

## 1. Verdict (honest)

**SPEC-091 wire-closure (2026-07-30):** acceptance binaries for IW0–IW5 are **CI-wired and locally green** (`make spec091-gates`). Full six-criterion **program DoD** (facade deleted, true kill-9/1M) remains open on C3 residuals.

Pre-program baseline that drove [19](19-improvement-plan.md) (kept for audit trail):

| # | Criterion | Grade (baseline) | One-line evidence (baseline) |
| --- | --- | --- | --- |
| C1 | Fully migrated out of dynamic tables (relational + AGE + vector) | **Partial** | KV retired (migration 125); chunk vectors typed but reads defaulted `legacy_tables`; entity/rel/report had no typed home |
| C2 | Tenant & workspace isolation preserved | **Partial (1 live defect)** | Malformed `X-Workspace-ID` matched any workspace |
| C3 | Technical debt removed | **Not met** | KV facade + permanent flags + SPEC-120 orphanage |
| C4 | Every query/CRUD benchmarked & optimal | **Partial** | Wave-0 scorecard unwired |
| C5 | Strong tests on the data layer | **Partial** | `e2e_spec091_*` manual-only |
| C6 | PG16/17/18 compatibility, best use of each | **Partial** | PR CI PG16-only |

**Post wire-closure (HEAD):** C2 D-defects closed; C5 **Met for wired suites**; C1 **Partial** (typed default + 130/131; operator must `--confirm-drop` 131); C4 **Partial→strong** (scorecard in CI; LD-06 policy note); C6 **Partial→strong** (capability health + pg16/pg18 smoke); C3 **Partial** (census allowlist, facade remains).

---

## 2. Method

Each criterion is graded from **code evidence at HEAD**, not from plan documents: file:line citations verified live, env-flag defaults treated as runtime truth, CI wiring checked in `.github/workflows/`, migration SQL read directly. A feature that exists but is flag-off, `#[ignore]`d, or CI-unwired is graded as **present, not enforced**. Grades: **Met** (true by default, enforced by a gate), **Partial** (true for some surfaces or true-but-unenforced), **Not met** (contradicted by evidence).

---

## 3. C1 — Migrated out of dynamic tables: **Partial**

*Why it matters: the spec's premise is one typed authority per family (LAW-D6) with schema owned by migrations (LAW-D5/LD-03). "Migrated out" means legacy relations are gone **and** no runtime path still depends on them.*

**What is done (Met):**
- Generic KV relations are physically gone post-migration: **125** drops all `eq_%_kv` + `eq_%_kv_stats` behind verified purge + durable-row guard (`migrations/125_spec091_kv_drop.sql:180-209`). Runtime KV DDL is removed — `PostgresKVStorage::initialize()` is a no-op (`kv.rs:240-252`).
- Typed SSOTs exist and are authoritative by default for: chunk text (`chunks`, authority default `relational`), dedup (`ingestion_dedup`, 107/117), quarantine (107), document shells (`documents`, 122), artifacts/checkpoints (116), LLM cache (`llm_cache`, 124) — full map in [16 §3](16-post-cutover-assessment.md).
- AGE remains the graph authority (LD-04); relational `entities`/`relationships` projections carry tenant/workspace uniqueness (001).
- Chunk vectors have a typed home: `chunk_embeddings` (108) with dual-write on ingest, engine backfill + verify, recall-parity e2e, and guarded retirement migration **126** (chunks only, `--confirm-drop`).

**What is not done (gaps):**
1. **Chunk-vector reads default to typed (wire-closure).** `EDGEQUAKE_VECTOR_BACKEND` unset → `TypedEmbeddings` (`vector_backend.rs`); explicit `legacy_tables` remains soak rollback (refused post-migration 126 by cutover guard).
2. **Entity/relationship/community-report vectors have no typed home at all.** Migration 126's own header scopes it to chunks (`migrations/126_spec091_vector_drop.sql:1-27`); these key shapes stay on `eq_*_vectors` with the full runtime DDL fleet: `create_table` + ALTER + index ensure (`vector/ddl.rs:202-259,306-324`), per-workspace table creation + dimension DROP/recreate (`workspace_vector.rs:97-124`). LD-03 remains violated for this fleet.
3. **The KV adapter is still a live runtime dependency.** Two instances are constructed at every boot (`state/postgres.rs:342-353`) and ~40 call-site files across api/query/core/pipeline still consume the `KVStorage` trait through a routing facade with 42P01 degrade shims (`kv.rs:271-451`).
4. **`get_by_ids` (unordered) has zero typed routing** (`kv.rs:332-367`). Its one production caller, document download (`handlers/documents/query/download.rs:57-68`), gets `Ok(vec![])` post-125 → **404 "Document not found"** — a live regression hazard, not just debt.
5. **Unknown key families silently lose writes post-drop**: unclassified keys still attempt raw KV writes (`family_mode_for_key` fail-safe `Kv`), which no-op with only a warning after 125.
6. **Typing is envelope-deep in the new sidecars**: `pipeline_checkpoints.payload`, `document_artifacts.payload`, `llm_cache.value`, `compensation_quarantine.payload` remain JSONB blobs (typed keys/FKs/lifecycle, untyped content).
7. **Legacy orphans survive 126**: `eq_*_vectors_stats` relations + `eq_*_vectors_stats_*` functions match neither drop pattern; `eq_hot_ann_workspaces` is still runtime-created (`vector/ddl.rs:624`).

**Grade justification:** the KV family is fully migrated (Met); chunk vectors are typed but not served (Partial); the vector fleet and the facade are not migrated (Not met). Net **Partial**, trending Met after IW2/IW3.

---

## 4. C2 — Tenant & workspace isolation preserved: **Partial (one live defect)**

*Why it matters: consolidation must not weaken isolation that the dynamic layer enforced by key-prefix convention; every new typed table and query path must carry the scope or provably inherit it.*

**What is strong:**
- Typed tables are scope-carrying: `documents`, `chunks`, `entities`, `relationships`, `tasks` (NOT NULL, 019), `ingestion_dedup` (UNIQUE on `(workspace_id, content_hash, pipeline_version)`, 107 — strongest scoping in the system), `chunk_embeddings` (NOT NULL + index, 108).
- Strict matchers are fail-closed: graph `properties_match_tenant_context` requires exact tenant AND workspace equality (`handlers/isolation.rs:44-108`); relational CRUD binds scope in SQL.
- Test existence: `e2e_tenant_isolation` (incl. header-spoofing attack), `e2e_postgres_workspace`, `e2e_postgres_rls`, `contract_workspace_scoped_analytics` (P-G12 no-leak pins), `task_scope` foreign-track-id 404.

**Defects and soft spots:**
1. **Live defect — malformed workspace header matches everything.** In `metadata_matches_workspace_context`, if the request's `X-Workspace-ID` fails UUID parsing the function `return true` (`edgequake/crates/edgequake-api/src/services/isolation_context.rs:87-90`) — legacy-alias document paths (list/dedup/delete) match documents in **any** workspace. The strict graph path is fail-closed; this asymmetry is the sharpest concrete bug found.
2. **Header wins over JWT by default.** `strict_tenant_bind: false` at both construction sites (`state/security_config.rs:40,60`); on JWT↔header mismatch `merge_claim_into_context` only logs and the header executes (`middleware.rs:418-454`). Master API keys map to `Role::Admin` with `jwt_tenant_id: None` (`auth_validation.rs:22-33`) — cross-tenant by design via spoofable headers.
3. **Task scoping is conditional on the header existing.** `get_task_for_context` checks `task.workspace_id` only when the request carries a parseable workspace UUID (`services/task_scope.rs:23-28`); a headerless caller who knows a `track_id` reads/cancels cross-workspace. `tenant_id` is never checked on this path.
4. **Headerless vector queries span all workspaces.** `query_execute.rs:150` passes `tenant_ctx.workspace_id.clone()` (None when absent) → the scope predicate is simply omitted (`chunk_embedding_index.rs:163-165`; legacy `storage_impl.rs:797-801`); only SPEC-031 `allowed_document_ids` mitigates, when computed.
5. **The RLS layer is inert in the shipped topology.** Fail-closed FORCE RLS (migration 096) protects only non-superuser roles; the product connects as the `edgequake` superuser, which bypasses RLS unconditionally. AGE-label RLS (081) is opt-in and fail-open when the GUC is unset. The `e2e_postgres_rls` suite tests a role the product never uses — and is fully ignored (C5).
6. **Scope columns that are never written or never documented**: `compensation_quarantine.workspace_id` exists (107) but the writer never sets it (`quarantine_sink.rs:46-57` — drain operates globally); `llm_cache` is cross-workspace by construction (PK `(cache_key, namespace)`, migration 124:28) — an undocumented cross-tenant data flow with zero test coverage.
7. **AGE graph is one global graph per namespace** with property-level scoping; `LegacyNullAsWildcard` (`graph/helpers/vertex_filter.rs:14-17`) deliberately matches NULL-scoped vertices/edges for any tenant (cascade/viewer paths) and has **no Postgres-backed leakage test**.

**Grade justification:** strict relational/typed paths are genuinely strong, but criterion says "preserved" — a malformed-header bypass (#1), conditional task/vector scoping (#3/#4), and a fully inert defense-in-depth layer (#5) mean isolation is **Partial**, with GAP-091-08 rated a defect to fix before any closure claim.

---

## 5. C3 — Technical debt removed: **Not met**

*Why it matters: the refactor's promise was to delete the dynamic layer, not to wrap it. Debt that outlives its migration becomes permanent surface area.*

1. **The facade outlived the store.** ~40 call-site files still speak `KVStorage` APIs routed through `kv.rs` (§3.3); the 42P01 degrade shims make the facade un-deletable until callers migrate to typed ports (LD-05 partial).
2. **Permanent rollback surface.** Family flags, `EDGEQUAKE_CHUNK_TEXT_AUTHORITY`, `EDGEQUAKE_VECTOR_BACKEND` remain env-settable to `kv`/`dual`/`legacy_tables` forever — LD-07 intended flags as temporary scaffolding; nothing schedules their retirement.
3. **Per-row dual-write loop.** `dual_write_shell_upserts` issues one INSERT per shell key plus two LEFT JOINs on the metadata arm (`document_shell.rs:159-258`) — violates the project's own LAW-D7 batching that sibling modules honor (`llm_cache.rs` uses one `unnest` per batch).
4. **Dead code in the tree.** `PostgresKeywordCache` creates `eq_{ns}_keyword_cache` at runtime but has **no production constructor call** — the engine uses `InMemoryKeywordCache` (`edgequake-query/src/engine_impl/mod.rs:435`); `eq_hot_ann_workspaces` runtime registry (`vector/ddl.rs:624-657`); kv-kind row-count stats triggers dormant (`row_count_stats.rs`).
5. **Drain without an applier.** `EDGEQUAKE_COMPENSATION_DRAIN=on` is coerced to `dry-run` with a **noop applier** at boot (`state/postgres.rs:283-302`) — quarantine dead-letters accumulate without remediation in production.
6. **SPEC-120 orphanage.** `handlers/operations.rs`, `services/fenced_write.rs`, `services/cancel_notify.rs`, `services/document_stage_mirror.rs`, `handlers/operation_presentation.rs` exist but are **untracked and never wired into crate roots**; their 8 contract tests are quarantined in `tests-wip-spec120-capacity/` (README: restoration = wire modules + routes + expand `TaskStatus`).
7. **Ornamental flag.** Runtime routes both `doc:hash:` and `staging:hash:` through the DOC_HASH family — `EDGEQUAKE_KV_FAMILY_STAGING_HASH` is largely decorative ([16 §3](16-post-cutover-assessment.md)).
8. **Fence default off** (`serving_fence`): LD-09's fail-closed visibility contract ships opt-in; at scale the fence remains advisory.

**Grade justification:** every debt item above has a file:line address and none is scheduled for deletion — **Not met**. This is the criterion with the largest gap between "migration done" and "debt removed."

---

## 6. C4 — Benchmarked & optimal queries: **Partial**

*Why it matters: LAW-D7/D8 promise batch-first, scale-off-request-path access; a typed layer that is correct but unmeasured can regress silently under the first large corpus.*

**What is genuinely benchmarked (legacy surfaces):**
- KV: p95 budgets + EXPLAIN ANALYZE plan assertions (`e2e_spec061_kv_access_perf.rs`: upsert < 100ms, get_by_ids < 50ms, prefix < 100ms, count < 20ms).
- Vector ANN (legacy `eq_*_vectors`): scaling ladders, recall pareto, filtered-recall gates, partial-HNSW EXPLAIN, iterative-scan GUC contracts (`e2e_spec064..078` suite).
- Graph: expand/degrees p95 budgets, 100k-node gate (`e2e_spec066_graph_g1.rs`).
- Tasks: claim CTE is bounded-sample, index-annotated (`edgequake-tasks/src/postgres.rs:638-648`); chaos/budget contracts exist.
- Harnesses: `tests/support/perf_harness.rs` (p95 + hard budgets + JSONL CI scraping), `assert_plan_uses_index`, SPEC-090 statement-timeout contracts.

**What is not benchmarked (the new data layer itself):**
1. **No perf gate covers the SPEC-091 typed tables.** The Wave-0 baseline scorecard exists only as spec text (`00-raw-needs.md:424-519`: filtered ANN p95 < 150ms w/ recall gate, ingest tx p95 < 2s, pool acquisition p95 < 10ms); the planned binaries (`e2e_spec091_ingestion_p95_budget`, `e2e_spec091_retrieval_slo_protection`) **do not exist** ([11](11-e2e-test-matrix.md) marks them planned).
2. **`chunk_embeddings` has no ANN index.** Search is exact `ORDER BY embedding <=> $1::halfvec LIMIT $4` over a btree-prefiltered set (`chunk_embedding_index.rs:157-167`), plus a `find_model_id` round trip per search (`:150`). Model-scoped partial HNSW is planned (spec 06), not landed.
3. **Index gaps on hot typed paths**: documents listing filters `workspace_id = $1 ORDER BY created_at DESC` with no composite (`document_read_model.rs:172-176`); `shell_staging_keys` is a LIMIT-less expression scan (`document_shell.rs:330-336`); workspace-scoped delete uses an index-defeating `OR metadata->>'workspace_id'` (`document_read_model.rs:402-406`).
4. **ef_construction is still triplicated** — 32 (migration 071) / 128 (`config.rs`) / 64 (`docker/init.sql`) — LD-06's single benchmarked value was never produced; recall parity was measured on a 40-row corpus (`e2e_spec091_recall_parity`).

**Grade justification:** the *measurement culture* is real but it belongs to the old layer; the new layer inherits none of it yet — **Partial**.

---

## 7. C5 — Strong tests on the data layer: **Partial**

*Why it matters: an unwired test is a document, not a gate. "Strong" = exists + deterministic + enforced in CI.*

**Strong (authorship):**
- ~350 integration test files across 7 crates. The W3 chunk-embedding suite is textbook: port conformance (`e2e_spec091_chunk_embeddings`), recall@10 parity vs exact baseline, dual-backend parity + fallback counter, crash-resume backfill, guarded retire (`e2e_spec091_vector_retire`).
- Wave-D end-state contracts are deterministic (0 sleeps) and fixture-isolated (`e2e_spec091_wave_d`, `e2e_spec091_console`, `support/spec091_fixture.rs`).
- Boot gate has a real contract suite (`contract_spec091_boot_gate`: exit-78 pins, downgrade protection, stale-flag shim, health agreement).
- AGE graph has genuine CI teeth: `postgres-age-tests` runs `storage_backend_contract`, `backend_e2e_contract`, `graph_isp_contract`, `spec022_cypher_prepared_postgres`, plus a true concurrent source_ids race test.
- Queue: SKIP LOCKED claim/lease e2e, hermetic chaos (EC-17..24), dual-pool exactly-once.

**Weak (enforcement):**
1. **Zero CI wiring for the SPEC-091 suite.** `rg "spec091|cli_migrate|upgrade.soak" .github/workflows/` finds nothing — every `e2e_spec091_*`, the boot gate, the CLI console test, and `make spec091-upgrade-soak` are manual-only. CI could regress all of them tomorrow without going red.
2. **The main postgres job is disabled.** `postgres-integration.yml:49` carries `if: false # TEMPORARILY DISABLED: Failing due to missing AGE extension` — taking RLS, workspace, tasks-PG, and postgres-integration coverage down with it; the AGE image it lacked now exists (`ghcr.io/raphaelmansuy/edgequake-postgres`).
3. **RLS suite: 11/11 tests `#[ignore]`d** (`e2e_postgres_rls.rs`) and it tests a role the app never connects as (C2.5).
4. **No property/fuzz testing anywhere**: zero matches for proptest/quickcheck/arbitrary across the repo. Prime targets: chunk writer key handling, Cypher query builder, migration cursor/keyset logic, dedup hash keys.
5. **No process-kill chaos**: `chaos_spec091_*` binaries don't exist; crash coverage is graceful-resume only (`e2e_spec091_vector_backfill_crash_resume`), not kill -9 mid-batch (EC-01/R-04).
6. **No scale proofs in any gate**: 10k+ document typed soak and the 1M-chunk delete residue proof (M-4.2) are planned, not binary; multi-replica lease storm (EC-02), concurrent-ingest-during-backfill (EC-03), failed-migration rollback — all planned.
7. **SPEC-120 WIP quarantine** (see C3.6): 8 contract tests not compiled, their modules orphaned.

**Grade justification:** authorship is strong; enforcement is the weakest link across all six criteria — **Partial** (and the primary reason no other criterion can claim durable closure).

---

## 8. C6 — PG16 / PG17 / PG18, best use of each: **Partial**

*Why it matters: the spec pins three supported majors; "best use of each" means version-differential value, not merely "runs on all three."*

**Compatibility engineering (strong):**
- Triple-track pinned images with per-major AGE branches — single SSOT `docker/extension-pins.sh:22-38` (pg16: AGE 1.6.0 / pg17: 1.7.0 / pg18 default: 1.8.0-rc0; pgvector **v0.8.5** everywhere).
- Runtime capability probing instead of version assumption: `capabilities.rs:203-223` probes `server_version_num`, gates `uuidv7()` on PG ≥ 18 with `Uuid::new_v4()` fallback (`id_allocation.rs:8-21`); pgvector iterative scan gated on extversion ≥ 0.8 (`vector/search_tuning.rs:143-183`); CVE floor 0.8.2 (`capabilities.rs:33-37`).
- Migrations are version-unconditional SQL — no `server_version` DO-blocks; the same train applies identically on all three majors.
- Release CD publishes `:VERSION-pg16/pg17/pg18` image tracks.

**Best-use gaps:**
1. **PG17 extracts zero differential value.** Identical SQL path to PG16; its only distinction is the AGE 1.7 image pin — an extension property, not a PG17 feature. PG17 is also the least-tested profile in PR CI.
2. **PG18 exploits only `uuidv7()`.** Async I/O (`io_method`), skip-scan-aware index design, virtual generated columns, `RETURNING OLD/NEW` (cited as a target for the LAW-D3 outbox in [02](02-first-principles.md)), and `NOT NULL`-from-validated-CHECK are all documented aspirations: `docs/data-layer/version-matrix.md` is a pending ledger (236 "pending" rows), not code.
3. **PR CI is single-version.** Every PR gate pins `pgvector/pgvector:pg16` (`spec013-proof-pr.yml:30`, `migration-guard.yml:86`); the `[pg16, pg17, pg18]` matrix exists but is schedule/battle-gated (`postgres-matrix-nightly.yml:76,82,140`) — PG17/PG18 breakage would be found nightly at best.
4. **Doc drift**: `Makefile:1562,1569` and `docker/README.md:14-16,103` still advertise "pgvector 0.8.3" against the 0.8.5 SSOT pin.

**Grade justification:** compatibility: **Met** (well-engineered, security floors, graceful degradation). "Making best use of each": **Not met** for PG17, one-feature for PG18. Net **Partial**.

---

## 9. Gap register (GAP-091-*)

Severity: **D** defect (wrong behavior now) · **H** high (blocks a criterion) · **M** medium (debt/enforcement).

| ID         | Gap                                                                                         | Criterion | Severity | Evidence                                                                        | Owning wave                |
| ------------| ---------------------------------------------------------------------------------------------| -----------| ----------| ---------------------------------------------------------------------------------| ----------------------------|
| GAP-091-01 | KV routing facade live for ~40 call sites; 42P01 shims permanent                            | C1/C3     | H        | `kv.rs:271-451`, `state/postgres.rs:342-353`                                    | IW3                        |
| GAP-091-02 | Chunk-vector reads default `legacy_tables`; typed serving opt-in                            | C1        | H        | `vector_backend.rs:13-27`                                                       | IW2                        |
| GAP-091-03 | Entity/rel/report vectors: no typed home, full runtime DDL fleet                            | C1        | H        | `vector/ddl.rs:202-259`, `workspace_vector.rs:97-124`, mig 126 header           | IW2                        |
| GAP-091-04 | `get_by_ids` unrouted → download 404 post-125                                               | C1        | **D**    | `kv.rs:332-367`, `download.rs:57-68`                                            | IW0                        |
| GAP-091-05 | JSONB remains primary payload in sidecars (envelope-deep typing)                            | C1        | M        | migrations 116/124                                                              | IW3 (accepted, documented) |
| GAP-091-06 | `eq_*_vectors_stats` + triggers orphaned by 126; `eq_hot_ann_workspaces` runtime-created    | C1/C3     | M        | `vector/ddl.rs:624-657`                                                         | IW2                        |
| GAP-091-07 | Unknown KV key families silently no-op post-drop                                            | C1        | M        | `kv.rs` family fail-safe `Kv`                                                   | IW0 (loud error)           |
| GAP-091-08 | Malformed `X-Workspace-ID` → matches any workspace                                          | C2        | **D**    | `isolation_context.rs:87-90`                                                    | IW0                        |
| GAP-091-09 | `strict_tenant_bind=false`: header wins over JWT; master key header-trust                   | C2        | H        | `security_config.rs:40,60`, `middleware.rs:418-454`, `auth_validation.rs:22-33` | IW0                        |
| GAP-091-10 | Task scoping skipped when no workspace header; tenant never checked                         | C2        | H        | `task_scope.rs:23-28`                                                           | IW0                        |
| GAP-091-11 | Headerless vector queries span all workspaces                                               | C2        | H        | `query_execute.rs:150`, `chunk_embedding_index.rs:163-165`                      | IW0                        |
| GAP-091-12 | RLS inert (superuser connection); AGE-label RLS opt-in + fail-open                          | C2        | M        | migration 096/081; `e2e_postgres_rls`                                           | IW0 (decision recorded)    |
| GAP-091-13 | `compensation_quarantine.workspace_id` never written                                        | C2        | M        | `quarantine_sink.rs:46-57`                                                      | IW0                        |
| GAP-091-14 | `llm_cache` cross-workspace sharing undocumented, untested                                  | C2        | M        | migration 124:28                                                                | IW0 (document + test)      |
| GAP-091-15 | AGE single global graph; `LegacyNullAsWildcard` untested for leakage                        | C2        | M        | `vertex_filter.rs:14-17`                                                        | IW5                        |
| GAP-091-16 | Shell dual-write N+1 per-row loop                                                           | C3/C4     | H        | `document_shell.rs:159-258`                                                     | IW1                        |
| GAP-091-17 | Dead code: `PostgresKeywordCache`, `eq_hot_ann_workspaces`, kv-kind stats                   | C3        | M        | `engine_impl/mod.rs:435`, `ddl.rs:624`                                          | IW3                        |
| GAP-091-18 | Compensation drain has no production applier (`on`→noop)                                    | C3        | H        | `state/postgres.rs:283-302`                                                     | IW3                        |
| GAP-091-19 | Rollback flags permanent (family/authority/backend)                                         | C3        | H        | flag envs passim                                                                | IW3                        |
| GAP-091-20 | SPEC-120 orphaned modules + 8 quarantined tests                                             | C3/C5     | H        | `tests-wip-spec120-capacity/README.md`                                          | IW3                        |
| GAP-091-21 | Fence default off; STAGING_HASH flag ornamental                                             | C3        | M        | [16 §3-4](16-post-cutover-assessment.md)                                        | IW3 (measurement-gated)    |
| GAP-091-22 | Wave-0 perf scorecard spec'd, never executed on typed tables                                | C4        | H        | `00-raw-needs.md:424-519`, [11](11-e2e-test-matrix.md)                          | IW1                        |
| GAP-091-23 | No HNSW on `chunk_embeddings` (exact scan)                                                  | C4        | M        | `chunk_embedding_index.rs:157-167`                                              | IW1 (measurement-gated)    |
| GAP-091-24 | Index gaps: listing composite, staging scan, OR-predicate delete                            | C4        | M        | `document_read_model.rs:172-176,402-406`, `document_shell.rs:330-336`           | IW1                        |
| GAP-091-25 | ef_construction triplicated 32/128/64 (LD-06 unconverged)                                   | C4        | M        | mig 071 / `config.rs` / `init.sql`                                              | IW1                        |
| GAP-091-26 | Zero CI wiring for `e2e_spec091_*` / boot gate / soak                                       | C5        | H        | `.github/workflows/` (no matches)                                               | IW0                        |
| GAP-091-27 | `postgres-tests` job `if: false`; RLS suite 11/11 `#[ignore]`                               | C5        | H        | `postgres-integration.yml:49`, `e2e_postgres_rls.rs`                            | IW0                        |
| GAP-091-28 | No property/fuzz tests (0 proptest matches)                                                 | C5        | M        | repo-wide grep                                                                  | IW5                        |
| GAP-091-29 | No kill-9 chaos binaries                                                                    | C5        | M        | no `chaos_spec091_*` files                                                      | IW5                        |
| GAP-091-30 | No 10k+/1M scale proofs in gates; planned matrix rows lack binaries                         | C5        | H        | [11](11-e2e-test-matrix.md) planned rows                                        | IW5                        |
| GAP-091-31 | PG17: zero differential exploitation                                                        | C6        | M        | identical SQL path; nightly-only matrix                                         | IW4                        |
| GAP-091-32 | PG18: only uuidv7; async I/O / skip scan / virtual gen cols / RETURNING OLD/NEW unexploited | C6        | M        | `capabilities.rs:222`, `version-matrix.md`                                      | IW4                        |
| GAP-091-33 | PR CI single-version (pg16); full matrix nightly-only                                       | C6        | H        | `spec013-proof-pr.yml:30`, `postgres-matrix-nightly.yml:76,82`                  | IW4                        |
| GAP-091-34 | Stale "pgvector 0.8.3" docs vs 0.8.5 pin                                                    | C6        | M        | `Makefile:1562,1569`, `docker/README.md:14-16,103`                              | IW4                        |

---

## 10. What "done" means — falsifiable closure per criterion

| Criterion | Closure definition (all must hold, CI-enforced) |
| --- | --- |
| **C1** | Post-migration-127: `count(*)` of `eq_%_kv` + `eq_%_vectors` relations = 0; boot performs zero storage DDL (contract test asserts no CREATE/ALTER/DROP path reachable outside migrations); vector reads typed-only (no legacy flag); unknown key family = loud error, never silent no-op |
| **C2** | Malformed/absent scope headers are fail-closed (contract tests on PG for documents/chunks/tasks/vectors/graph); task read requires matching scope; quarantine rows carry `workspace_id`; cross-workspace AGE + ANN leakage tests green in CI; RLS decision recorded (non-superuser deployment wired, or documented acceptance with app-layer proof suite) |
| **C3** | `KVStorage` facade deleted (zero trait imports outside typed ports); GAP-091-17 dead-code list removed; drain has a real applier or quarantine redesigned; rollback flags retired (advisor refuses stale values, LD-14 pattern); SPEC-120 modules wired + tests green or explicitly descoped per-test |
| **C4** | Wave-0 scorecard executable and gating in CI (ingest p95 < 2s, filtered ANN p95 < 150ms w/ recall, pool acquisition < 10ms); single benchmarked `ef_construction` (LD-06 closed); `chunk_embeddings` ANN index decision backed by a measured ladder; listing/staging-scan queries index-backed (EXPLAIN gates) |
| **C5** | Every `e2e_spec091_*` + boot gate + soak wired into a workflow that runs on data-layer PRs; RLS + workspace isolation suites re-enabled; proptest suites for key grammar/chunk writer/Cypher builder; `chaos_spec091_*` kill-9 binaries in nightly; 10k-doc soak + 1M delete residue on schedule |
| **C6** | Capability matrix surfaced at runtime (`/health` + console); PR smoke on pg16 + pg18 (full matrix nightly); each adopted version feature cites a benchmark (LAW-I2); docs pins match `extension-pins.sh` (drift test) |

---

## Related

- Improvement program: [19-improvement-plan.md](19-improvement-plan.md) (waves IW0–IW5)
- HEAD wave audit: [16-post-cutover-assessment.md](16-post-cutover-assessment.md) · Boot gating: [17-boot-migration-gating.md](17-boot-migration-gating.md)
- Planned-vs-exists tests: [11-e2e-test-matrix.md](11-e2e-test-matrix.md) · Perf targets: [00-raw-needs.md](00-raw-needs.md) §scorecard · Version ledger: [docs/data-layer/version-matrix.md](../../docs/data-layer/version-matrix.md)
