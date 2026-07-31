# 04 — Cross-Reference Matrix & ID Registry

> Single source of truth for every identifier used in SPEC-091. When any document references an ID, its definition lives here (SSOT). Code paths in the findings table were verified at pin `36c45b7`; HEAD deltas and post-cutover SSOT live in [16-post-cutover-assessment.md](16-post-cutover-assessment.md).

## ID namespaces

| Prefix | Meaning | Defined in |
| --- | --- | --- |
| `LAW-D1..D8` | First-principles laws (data layer) | [02-first-principles.md](02-first-principles.md) |
| `LAW-P1..P5` | Performance laws | [08-performance-contract.md](08-performance-contract.md) |
| `LAW-Q1..Q7` | First-principles laws (queue & admission) | [12-queue-admission-first-principles.md](12-queue-admission-first-principles.md) |
| `LAW-C1..C6` | Console advisor laws | [15-migration-console-cli.md](15-migration-console-cli.md) |
| `LAW-B1..B5` | Boot migration-gating laws | [17-boot-migration-gating.md](17-boot-migration-gating.md) |
| `LAW-I1..I6` | Improvement/closure laws | [19-improvement-plan.md](19-improvement-plan.md) §2 |
| `LD-01..LD-17` | Locked decisions | [README.md](README.md#locked-decisions) |
| `F-091-01..20` | Findings (current-state, verified in code) | this file § Findings |
| `GAP-091-01..34` | Six-criteria completeness gaps (HEAD) | [18-full-completeness-assessment.md](18-full-completeness-assessment.md) §9 |
| `R-01..R-30` | Risks (forward + realized Wave D / QW) | [09-risk-register.md](09-risk-register.md) |
| `EC-01..EC-36` | Edge cases | [10-edge-cases.md](10-edge-cases.md) |
| `R-B1..R-B5` | Boot migration-gating risks | [17-boot-migration-gating.md](17-boot-migration-gating.md) §9 |
| `EC-B1..EC-B14` | Boot migration-gating edge cases | [17-boot-migration-gating.md](17-boot-migration-gating.md) §8 |
| `R-I1..R-I6` | Improvement-program risks | [19-improvement-plan.md](19-improvement-plan.md) §6 |
| `EC-I1..EC-I8` | Improvement-program edge cases | [19-improvement-plan.md](19-improvement-plan.md) §5 |
| `M-1.x / M-4.x / M-5.x` | Scorecard measures | [08-performance-contract.md](08-performance-contract.md) |
| `W0..W5` | Planned implementation waves (data layer) | [06-implementation-plan.md](06-implementation-plan.md) |
| `Waves A–D` | Informal KV-retirement execution waves (≈ W0–W2 contract) | [06](06-implementation-plan.md), [16](16-post-cutover-assessment.md) |
| `C0..C3` | Migration console delivery waves | [15-migration-console-cli.md](15-migration-console-cli.md) |
| `QW0..QW3` | Implementation waves (queue & admission) | [14-queue-admission-plan.md](14-queue-admission-plan.md) |
| `B0..B2` | Boot migration-gating waves (spec → gate → env alignment) | [17-boot-migration-gating.md](17-boot-migration-gating.md) §7 |
| `IW0..IW5` | Improvement waves (six-criteria closure program) | [19-improvement-plan.md](19-improvement-plan.md) §4 |
| `e2e_spec091_*`, `contract_spec091_*`, `chaos_spec091_*` | Tests | [11-e2e-test-matrix.md](11-e2e-test-matrix.md) |
| `S-01..S-13` | Raw-study finding IDs (traceability only) | [00-raw-needs.md](00-raw-needs.md) |
| Doc **16** | HEAD post-cutover assessment (SSOT for working-tree truth) | [16-post-cutover-assessment.md](16-post-cutover-assessment.md) |
| Doc **18** | Six-criteria full-completeness assessment + GAP register | [18-full-completeness-assessment.md](18-full-completeness-assessment.md) |
| Doc **19** | Improvement plan (IW0–IW5) closing the six criteria | [19-improvement-plan.md](19-improvement-plan.md) |

## Findings register (S-XX ↔ F-091-XX ↔ law ↔ code ↔ wave ↔ test)

| F-091 | S-XX | Finding (law violated) | Primary code locus | Wave | Primary test |
| --- | --- | --- | --- | --- | --- |
| F-091-01 | S-01 | Orphaned retrieval unit: chunk↔embedding presence not enforceable (LAW-D1) | `adapters/postgres/vector/ddl.rs:206-216` (TEXT PK, no FK target) | W3, W4 | `e2e_spec091_serving_fence_fail_closed` |
| F-091-02 | S-02 | Unpopulated relational text authority (LAW-D6) | `edgequake-pipeline/src/chunk_storage.rs:1,12-22`; `migrations/001_init_database.sql:205-219` | W1 | `e2e_spec091_backfill_coverage_100` |
| F-091-03 | S-03 | Incompatible identifier types uuid↔text (LAW-D2) | `kv_key_schema.rs:66` vs `001_init_database.sql:206` | W1–W3 | `contract_spec091_chunk_repository_identity` |
| F-091-04 | S-04 | Dynamic schema fleet: runtime vector DDL, six swallowed ALTERs (LAW-D5) | `adapters/postgres/vector/ddl.rs:267-285` | W3 | `e2e_spec091_single_schema_generation` |
| F-091-05 | S-05 | Compensation quarantine without guaranteed drain (LAW-D3/D4) | `edgequake-storage/src/compensation.rs:83-123` | W4 | `e2e_spec091_quarantine_drain_slo` |
| F-091-06 | S-06 | `HnswRuntimePolicy::default()` = Full vs env default Half (LAW-D6 of defaults) | `adapters/postgres/hnsw_runtime_policy.rs:25-40` | W0 (trivial fix) | `contract_spec091_hnsw_policy_default_half` |
| F-091-07 | S-07 | Hot-workspace index churn at 1,000 rows (LAW-D8) | `adapters/postgres/vector/ddl.rs:532-608` (`partial_min_rows`) | W5 | `e2e_spec091_one_hnsw_membership` |
| F-091-08 | S-08 | Cached-count drift across three counter kinds (LAW-D4) | `adapters/postgres/row_count_stats.rs:50-223`; `workspace_ops.rs:444-454` | W1, W4 | `e2e_spec091_counts_are_projections` |
| F-091-09 | S-09 | Dormant CQRS read model (`entity_sync_mode=disabled`) (LAW-D6 labeling) | `migrations/039_cqrs_entities_schema.sql`; `040` backfill marker | W4 | `e2e_spec091_dormant_read_model_labeled` |
| F-091-10 | S-10 | Runtime-created KV relations + discovery by `LIKE` (LAW-D5) | `adapters/postgres/kv.rs:107-155`; `migrations/068` | W2 | `e2e_spec091_kv_fleet_zero_readers` |
| F-091-11 | S-11 | Statistics computed from an empty relation; two facts, one expression (LAW-D4) | `edgequake-core/src/workspace_service_impl/workspace_ops.rs:448,451` | **Pre-W1 patch** | `e2e_spec091_workspace_stats_truthful` |
| F-091-12 | S-12 | Legacy 8-hex workspace slugs (32-bit entropy) | `traits/workspace_vector.rs:105-107` | W3 | `e2e_spec091_legacy_slug_retired` |
| F-091-13 | S-13 | Three competing `chunks` definitions (001/002/init.sql + view) | `migrations/001:205-219`, `migrations/002:51`, `edgequake/docker/init.sql:96-119,562-563` | W0 | `e2e_spec091_single_schema_generation` |
| F-091-14 | — (new, C-5) | `ef_construction` triplicated: 32 (071) / 128 (`config.rs:96-114`) / 64 (`init.sql:562-563`) (LAW-D5) | all three loci | W3 (benchmark), W0 (freeze) | `e2e_spec091_hnsw_policy_converged` |
| F-091-15 | — (new, C-1) | Ingestion persistence reachable via two entry paths; writer logic must exist in exactly one shared place (DRY) | `edgequake-pipeline/src/persistence/ingestion_persister.rs:282-298`; callers `edgequake-core/src/orchestrator/ingestion.rs:253`, `edgequake-api/src/services/ingestion_persist.rs:182` | W1 | `contract_spec091_single_chunk_writer` |
| F-091-16 | — (new, C-6) | No retirement ledger for legacy slug/vector relations (LAW-D5) | `adapters/postgres/workspace_table.rs:8-77` | W3 | `e2e_spec091_schema_generation_ledger` |
| F-091-17 | — (queue study) | Task status transitions ad-hoc: methods + raw SQL + field mutation; no transition table; `Failed` ambiguously retryable and terminal (LAW-Q2) | `edgequake-tasks/src/types/task.rs:191-302`; `postgres.rs:653-853`; `worker.rs:792-794`; `edgequake-api/src/services/orphan_task_recovery.rs:155-217` | QW0 | `contract_spec091_state_machine_transitions` |
| F-091-18 | — (queue study) | Provider concurrency guard is process-local and per-task, not cluster-global: 2 replicas ⇒ 2× provider load; five independent resolvers (LAW-Q3, LAW-Q1) | `edgequake-api/src/local_inference_gate.rs:15-18,81-120`; `edgequake-pipeline/src/pipeline/extraction.rs:41-43`; `pipeline/config.rs:177-306` | QW1, QW2 | `contract_spec091_provider_budget_acquire_release` |
| F-091-19 | — (queue study) | No enqueue-side admission: unbounded DB queue; 101st upload silently hangs on the 100-slot wake channel; `QueueMetrics.rate_limited` hardcoded `false` (LAW-Q4) | `edgequake-api/src/handlers/documents/upload/document_admission.rs:120`; `edgequake-tasks/src/queue.rs:84-94`; `postgres.rs:948`; wired `edgequake-api/src/state/postgres.rs:414` | QW2 | `e2e_spec091_queue_explicit_queued_state` |
| F-091-20 | — (queue study) | Tenant hard caps are the wrong unit for a provider-bound resource: caps under-feed a small budget and fail to protect against a large one (LAW-Q5) | `edgequake-tasks/src/tenant_limiter.rs:91-166,171-410`; `pipeline/config.rs:218-223` | QW3 | `e2e_spec091_queue_provider_budget_never_exceeded` |

## Laws ↔ findings ↔ decisions

| Law | Violated by | Honored by (keep) | Locked decision |
| --- | --- | --- | --- |
| LAW-D1 tuple integrity | F-091-01 | exact-reorder path | LD-09 |
| LAW-D2 single identity | F-091-03, F-091-12 | uuidv7 probe in `capabilities.rs:202-253` | LD-02 |
| LAW-D3 commit/fence | F-091-01, F-091-05 | idempotent compensation design | LD-09 |
| LAW-D4 counts as projections | F-091-08, F-091-11 | statement-level triggers (F-091-08 basis) | LD-01 |
| LAW-D5 one schema owner | F-091-04, F-091-10, F-091-13, F-091-14, F-091-16 | `_sqlx_migrations`, `edgequake_reconcile_state` digest ledger (migration 102) | LD-03, LD-06 |
| LAW-D6 one authority | F-091-02, F-091-06, F-091-09 | content_ref-only vector metadata contract | LD-01 |
| LAW-D7 batch-first | F-091-10 (key grammar) | `unnest` batch upserts (`storage_impl.rs:225`) | LD-05 |
| LAW-D8 scale off request path | F-091-07, F-091-08 (suffix scans) | CIC builds, partial-scan bounds | LD-08, LD-10 |
| LAW-Q1 capacity derived | F-091-18, F-091-20 | Makefile local/cloud profiles (kept as profile seeds) | LD-11 |
| LAW-Q2 single transition authority | F-091-17 | lease CAS discipline (`postgres.rs:793-822`) | LD-12 |
| LAW-Q3 cluster-global provider budget | F-091-18 | task `claim_next` SKIP LOCKED pattern (reused for slots) | LD-11 |
| LAW-Q4 bounded, honest queue | F-091-19 | `task_queue_pressure.rs` observability (kept, observational) | LD-12 |
| LAW-Q5 tenant fair-share | F-091-20 | park-not-churn machinery (`worker.rs:1017-1094`) | LD-13 |
| LAW-Q6 idempotent identity | — (honored today) | checksum dedup, single-flight registries, hash dedup | LD-12 |
| LAW-Q7 durable lifecycle intents | F-091-17 (convention, not law) | `cancellation.rs` intents, deletion cascade | LD-12 |

## Findings ↔ official documentation (July 2026)

| Finding | Official anchor |
| --- | --- |
| F-091-02, F-091-06 | TOAST out-of-line storage ([postgresql.org/docs/18/storage-toast](https://www.postgresql.org/docs/18/storage-toast.html)); stored generated columns ([ddl-generated-columns](https://www.postgresql.org/docs/18/ddl-generated-columns.html)) |
| F-091-03, F-091-12 | PG18 `uuidv7()` ([release-18](https://www.postgresql.org/docs/18/release-18.html)) |
| F-091-01, F-091-04 | pgvector multitenancy + filtered-ANN guidance ([github.com/pgvector/pgvector](https://github.com/pgvector/pgvector#multitenancy)) |
| F-091-08, F-091-11 | PG18 async I/O for scan/vacuum-heavy verification ([release-18](https://www.postgresql.org/docs/18/release-18.html)) |
| F-091-07 | pgvector: "filtering by few distinct values → partial index; many → partitioning" ([README § Filtering](https://github.com/pgvector/pgvector#filtering)) |
| F-091-09 | AGE releases: PG18/v1.8.0-rc0 + pg_upgrade helpers ([github.com/apache/age/releases](https://github.com/apache/age/releases)) |
| F-091-13, F-091-14, F-091-16 | PG18 transactional DDL; RLS ([ddl-rowsecurity](https://www.postgresql.org/docs/18/ddl-rowsecurity.html)) |

## Measures ↔ waves ↔ tests (summary; full matrix in 11)

| Measure | Gate | Wave | Test |
| --- | --- | --- | --- |
| M-1.1 backfill coverage = 100% of live KV chunk keys | Exit W1 | W1 | `e2e_spec091_backfill_coverage_100` |
| M-1.2 checksum mismatches = 0 for one ingestion cycle | Exit W1 | W1 | `e2e_spec091_dual_read_zero_divergence` |
| M-1.3 fallback reads = 0 for one release soak | Exit W1 | W1 | `e2e_spec091_fallback_counter_zero` |
| M-1.4 ingestion p95 inside W0 budget | Exit W1 | W1 | `e2e_spec091_ingestion_p95_budget` |
| M-2.1 `eq_*_kv` readers = 0, rows = 0 | Exit W2 | W2 | `e2e_spec091_kv_fleet_zero_readers` |
| M-3.1 recall@10 parity per migrated relation | Exit W3 | W3 | `e2e_spec091_recall_parity` |
| M-4.1 quarantine oldest age < 15 min | Exit W4 | W4 | `e2e_spec091_quarantine_drain_slo` |
| M-4.2 deletion residue = 0 @ 1M chunks | Exit W4 | W4 | `e2e_spec091_workspace_delete_zero_residue_1m` |
| M-5.1 exactly one active HNSW membership per generation | Exit W5 | W5 | `e2e_spec091_one_hnsw_membership` |

## Traceability rules

1. A finding may not enter a wave without a law, a code locus, and a test ID.
2. A measure may not gate a wave without a Wave-0 baseline value (sequencing invariant 1).
3. An `S-XX` reference is valid only as traceability to [00-raw-needs.md](00-raw-needs.md); new work cites `F-091-XX`.
4. Any code citation drifting more than one release behind the pin must be re-verified before its wave starts (the raw study's own method, retained).
