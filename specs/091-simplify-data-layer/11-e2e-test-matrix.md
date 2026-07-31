# 11 — E2E & Contract Test Matrix

> Naming: `e2e_spec091_*` (behavioral, full stack), `contract_spec091_*` (port conformance, per adapter), `chaos_spec091_*` (failure injection). Every test cites the finding(s)/edge case(s)/measure(s) it proves. All suites run in CI; L3/L4 ladder rungs run in the nightly perf pipeline.

## Pre-W1 patch (v0.22.1)

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_workspace_stats_truthful` | F-091-11: chunk_count and embedding_count are distinct, non-zero, sourced from real stores | ingest doc via API → stats endpoint returns truthful counts; regression-locks `workspace_ops.rs` fix |
| `contract_spec091_hnsw_policy_default_half` | F-091-06: `HnswRuntimePolicy::default()` == env-derived default (Half) | unit test pinning both constructors agree |

## Wave 0 — baseline & consolidation

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_single_schema_generation` | F-091-13: one `chunks` definition; init.sql derives from migrations; view columns match | schema diff assertion across `pg_attribute` for all three sources |
| `e2e_spec091_scorecard_recorded` | W0 exit: every M-0.x metric recorded with environment metadata | scorecard harness output validated for completeness |
| `e2e_spec091_legacy_slug_inventory_retire` | F-091-12/16, EC-12: inventory lists legacy + current relations; resolution prefers current | seed both table shapes; assert resolution order + inventory completeness |

## Wave 1 — text authority

| Test | Proves | Method |
| --- | --- | --- |
| `contract_spec091_single_chunk_writer` | F-091-15: core + API ingestion paths converge on one relational writer in `ingestion_persister.rs` | both entry paths produce identical `chunks` rows |
| `contract_spec091_chunk_repository_identity` | F-091-03: `ChunkId` is uuid end-to-end; no string keys cross the port | type-level + round-trip conformance |
| `e2e_spec091_backfill_coverage_100` | F-091-02, M-1.1: coverage = 100% of live well-formed KV chunk keys | seed KV, run descriptor, assert verification metrics |
| `e2e_spec091_dual_read_zero_divergence` | M-1.2: checksum mismatches = 0 for one ingestion cycle | dual-read comparator across randomized corpus |
| `e2e_spec091_fallback_counter_zero` | M-1.3: fallback reads = 0 for soak; counter is emitted | metrics assertion after flag=`relational` |
| `e2e_spec091_ingestion_p95_budget` | M-1.4: ingestion TX p95 within W0 budget with relational write added | perf harness vs baseline |
| `e2e_spec091_concurrent_ingest_during_backfill` | EC-03: re-ingestion during backfill leaves zero duplicates/gaps | concurrent writer + running job, then verification |
| `contract_spec091_backfill_malformed_keys` | EC-04, R-11: malformed keys quarantined, excluded from coverage denominator | seed bad keys; assert bucket + coverage math |
| `contract_spec091_oversize_chunk_guard` | EC-10: oversize text rejected + quarantined with typed error | boundary corpus |
| `e2e_spec091_giant_document_keyset` | EC-16: keyset cursor spans one document across batches; interrupted ANALYZE re-runs | 50k-chunk document; kill ANALYZE phase once |
| `chaos_spec091_crash_mid_batch` | EC-01, R-04: kill -9 mid-batch → resume loses ≤1 batch, zero duplicate effect | process kill at random batch boundary, diff post-resume |

## Wave 2 — KV removal

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_kv_fleet_zero_readers` | F-091-10, M-2.1: zero readers, zero rows per `eq_*_kv` before drop | pg stat + reader instrumentation assertion |
| `e2e_spec091_kv_family_isolation` | W2 mechanism: one family migration fails without affecting others | forced failure on one family; others green |
| `e2e_spec091_restore_point_recorded` | EC-14: drop job carries recorded recovery-point id; surfaces mark irreversibility | ledger + surface assertions |

## Wave 3 — typed embeddings

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_recall_parity` | M-3.1, R-05: recall@10 parity vs exact search per migrated relation | brute-force baseline comparison per relation |
| `e2e_spec091_schema_generation_ledger` | F-091-16: per-relation progress resumable after interruption | interrupt mid-relation; resume; assert ledger |
| `e2e_spec091_expand_contract_dimension_change` | EC-07: model change creates new generation; no vector loss; old retired post-gate | two model generations through full cycle |
| `e2e_spec091_extension_floor_refusal` | EC-06: preflight refuses below pgvector/AGE floors with capability report | faked extversion below floor |
| `e2e_spec091_hnsw_policy_converged` | F-091-14, LD-06: single ef_construction value in every indexdef fleet-wide | `pg_indexes` sweep equals ledger value |
| `e2e_spec091_legacy_slug_retired` | F-091-12: legacy relations retired via ledger; none created since cutover | fleet sweep |

## Wave 4 — serving lifecycle

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_serving_fence_fail_closed` | F-091-01, LD-09: chunk lacking embedding/graph/readiness is never query-visible | inject partial projections; assert invisibility |
| `e2e_spec091_quarantine_drain_slo` | F-091-05, M-4.1: DLQ oldest age < 15 min; terminal failures reported | inject compensation failures; watch drainer |
| `e2e_spec091_workspace_delete_zero_residue_1m` | M-4.2: deletion at 1M chunks leaves zero residue across relational/vector/AGE | L3 rung; sweep all stores post-delete |
| `e2e_spec091_workspace_delete_during_migration` | EC-05: deletion mid-job reconciles; cursor re-validates | delete workspace while job runs |
| `e2e_spec091_dormant_read_model_labeled` | EC-13, F-091-09: stats never read dormant read model as truth; freshness labeled | `entity_sync_mode=disabled` deployment |
| `e2e_spec091_counts_are_projections` | F-091-08, LAW-D4: counts trace to `chunks`/`chunk_serving_state`; no third opinion | cross-check all count surfaces |

## Wave 5 — measured scaling

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_one_hnsw_membership` | M-5.1, F-091-07: exactly one active ANN index per embedding generation | indexdef sweep during/after workspace promotion |
| `e2e_spec091_partition_gate_evidence` | LD-10: partitioning lands only with reproduced threshold breach attached | evidence artifact required in PR; CI checks link |

## Migration engine (cross-wave)

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_migration_lease_exclusivity` | EC-02: 10 instances ⇒ exactly 1 running job per step | concurrent boot storm |
| `e2e_spec091_progress_monotonic` | progress honesty: `completion_pct` non-decreasing across restart; ETA error < 20% final decile | restart mid-run; sample series |
| `e2e_spec091_retrieval_slo_protection` | EC-09: retrieval p95 degrades ≤10%; job pauses before breach | load generator + running job |
| `e2e_spec091_replica_lag_throttle` | EC-08: pause on lag; resume on recovery | lag simulator |
| `e2e_spec091_capacity_gate_shortfall` | EC-11: preflight refusal reports byte-exact shortfall | constrained volume |
| `e2e_spec091_empty_deployment_noop` | EC-15: fresh deployment completes all descriptors as no-ops | fresh DB boot |
| `chaos_spec091_lease_expiry_fencing` | stale lease holder cannot write after fencing token advances | pause instance past expiry; assert rejected writes |
| `chaos_spec091_descriptor_digest_mismatch` | changed descriptor refuses silent resume; requires new job id | mutate descriptor; attempt resume |

## Port conformance (runs per registered adapter, incl. in-memory)

`contract_spec091_ports::*` — idempotency under retry · partial-failure behavior · cursor stability under concurrent writes · deletion completeness · filter semantics · ordering guarantees · recall reporting (where declared) · **cost budget: no per-row round trips** (LAW-D7). An adapter failing any case is not shipped (LSP proven, LD-05).

## QW0 — state machine SSOT

| Test | Proves | Method |
| --- | --- | --- |
| `contract_spec091_state_machine_transitions` | F-091-17, LAW-Q2: every (state × event) cell of the transition table behaves as specified; illegal transitions return `TransitionError` | exhaustive matrix over 6 states × 9 events |
| `contract_spec091_state_machine_sql_guard_drift` | F-091-17: SQL claim/release guards encode the same table as the Rust `transition()` — cannot drift | assert `guard_sql(event)` fragment matches the table definition per event |
| `contract_spec091_state_machine_no_raw_mutation` | F-091-17: no `SET status` outside the SSOT module + the two claim/release SQL sites embedding `guard_sql` | source-scan test (`include_str!`, banned-string built at runtime) |

## QW1 — provider-slot ledger

| Test | Proves | Method |
| --- | --- | --- |
| `contract_spec091_provider_budget_acquire_release` | F-091-18, LAW-Q3: acquire/release/CAS semantics; budget never exceeded by N concurrent claimants | Postgres adapter vs seeded ledger, 2 simulated instances |
| `contract_spec091_provider_budget_reap_expired` | R-14: stale slots reclaimed by reaper; fencing token rejected after expiry | seed expired lease; reap; assert reuse + old-token release fails |
| `chaos_spec091_queue_worker_crash_lease_reclaim` | EC-22: kill -9 a worker holding task claim + slot; both reclaimed within TTL+ε | process kill at random stage; assert both ledgers converge |
| `e2e_spec091_queue_slot_granularity_stage` | R-13: one slot lease spans a pipeline stage, not per-call; acquire p95 < 5 ms | instrumented extraction stage on local profile |

## QW2 — admission & queued honesty

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_queue_explicit_queued_state` | F-091-19, EC-21, LD-12: saturate budget → upload returns 202 + `queue_position` + clamped ETA → drains FIFO-by-share; no handler blocks on the wake channel | load generator vs saturated local profile |
| `e2e_spec091_queue_eta_honest` | R-15: ETA error < 20% in the final decile; position always exact | recorded task series vs returned ETAs |
| `contract_spec091_admission_resolver_derives_all` | F-091-18, LAW-Q1: every cap in the system is `f(B)` from one resolver; legacy env vars map to budget overrides | table test over local/cloud/mock profiles |

## QW3 — fair-share & lifecycle edge cases

| Test | Proves | Method |
| --- | --- | --- |
| `e2e_spec091_queue_provider_budget_never_exceeded` | EC-20, F-091-20, LAW-Q3/Q5: N tasks × 2 tenants ⇒ inflight ≤ B at every instant; both tenants progress (no starvation) | concurrent harness sampling `provider_inflight` |
| `e2e_spec091_queue_single_tenant_full_budget` | R-16: single active tenant receives the full budget; drain rate ≥ 95% of pre-QW3 measurement | throughput A/B measurement |
| `e2e_spec091_queue_delete_while_processing` | EC-17, LAW-Q7: delete at each stage ⇒ cancel intent honored, cascade completes, zero residue, no orphan tasks | stage-parameterized delete × residue sweep |
| `e2e_spec091_queue_cancel_states` | EC-18: cancel from queued / mid-extraction / retry-backoff ⇒ `cancelled` terminal, no further claims | boundary-parameterized cancel |
| `e2e_spec091_queue_duplicate_while_processing` | EC-19, LAW-Q6: duplicate while queued/processing returns in-flight identity, zero new tasks; after failed ⇒ new task, same dedup identity | race harness + task-count assertion |
| `chaos_spec091_queue_shutdown_drain` | EC-23: shutdown with queued + in-flight tasks ⇒ drain budget honored, no lost or double-run tasks on restart | restart + task-ledger diff |
| `chaos_spec091_queue_provider_stall` | EC-24: hung provider ⇒ timeouts fire, breaker trips at 3 no-progress timeouts, queue recovers on provider return | stall-injecting provider double |

## B1 — boot migration gating ([17-boot-migration-gating.md](17-boot-migration-gating.md))

| Test | Proves | Method |
| --- | --- | --- |
| `contract_spec091_boot_gate::fresh_db_refuses` | EC-B1, LAW-B2: server boot against a fresh DB refuses, exit 78, message names pending count + `edgequake migrate dry-run` + `edgequake migrate` + runbook | in-process `AppState` boot on scratch DB |
| `contract_spec091_boot_gate::behind_db_refuses_lists_pending` | EC-B2, LAW-B3: message lists the exact pending versions | partially-migrated scratch DB |
| `contract_spec091_boot_gate::newer_db_refuses` | EC-B3, LAW-B5: applied > embedded refuses with distinct downgrade message | seed `_sqlx_migrations` beyond embedded latest |
| `contract_spec091_boot_gate::boot_succeeds_after_cli_migrate` | LAW-B1: same scratch DB refuses → CLI-mode migrate → boot gate passes | two-phase boot on one scratch DB |
| `contract_spec091_boot_gate::stale_flag_warns_and_does_not_apply` | EC-B9: `EDGEQUAKE_ALLOW_BOOT_MIGRATE=1` warns but the gate still refuses; schema untouched | env-scoped boot attempt |
| `contract_spec091_boot_gate::health_reports_pending` | LAW-B3 surfacing: `/health` derivation exposes `schema.pending_count` + `migration_required` | unit-level on the derivation fn |

## IW — improvement waves ([19-improvement-plan.md](19-improvement-plan.md) §7; all **planned** unless listed under Exists today)

| Test | Wave | Proves | GAP |
| --- | --- | --- | --- |
| `contract_spec091_strict_scope_headers` | IW0 | LAW-I4: malformed/absent scope headers fail closed across documents/chunks/tasks/vector/graph | GAP-091-08/10/11 |
| `contract_spec091_get_by_ids_typed` | IW0 | unordered batch read typed-routed; download-404 regression dead | GAP-091-04 |
| `contract_spec091_unknown_family_loud` | IW0 | unknown KV key family errors loudly, never silent no-op | GAP-091-07 |
| `e2e_spec091_ingestion_p95_budget` | IW1 | LAW-I1/I2: ingest tx p95 < 2s on the typed writer (Wave-0 scorecard) | GAP-091-22 |
| `e2e_spec091_retrieval_slo_protection` | IW1 | filtered ANN p95 < 150ms + recall gate; pool acquisition p95 < 10ms | GAP-091-22 |
| `contract_spec091_shell_batch_write` | IW1 | LAW-D7: shell dual-write is one `unnest` round trip (EXPLAIN + p95) | GAP-091-16 |
| `e2e_spec091_hnsw_policy_converged` | IW1 | LD-06: one benchmarked `ef_construction` (32/64/128 ladder ≥100k vectors) | GAP-091-25 |
| `e2e_spec091_fleet_recall_parity` | IW2 | entity/rel/report typed recall ≥ legacy baseline (flip gate) | GAP-091-02/03 |
| `contract_spec091_zero_runtime_ddl` | IW2 | C1 closure: boot performs zero storage DDL; `eq_%_vectors` count = 0 | GAP-091-03/06 |
| `contract_spec091_no_kv_facade` | IW3 | compile-time census: zero `KVStorage` imports outside typed ports | GAP-091-01 |
| `e2e_spec091_pg_matrix_smoke` | IW4 | typed CRUD green on pg16 + pg18 PR smoke | GAP-091-33 |
| `contract_spec091_capability_health` | IW4 | `/health` capability matrix matches `capabilities.rs` probe | GAP-091-32 |
| `proptest_spec091_key_grammar` | IW5 | property invariants (key grammar + adaptive batch clamps) — **exists** | GAP-091-28 |
| `proptest_spec091_chunk_writer` / `_cypher` | IW5 | **Descoped (2026-07-30):** covered by existing port/conformance suites (`e2e_spec091_chunk_embeddings`, Cypher prepared contracts in AGE CI); no separate proptest binaries | GAP-091-28 |
| `chaos_spec091_crash_mid_batch` / `_lease_expiry_fencing` | IW5 | lease cancel/expiry fencing stand-ins (true kill -9 process chaos remains residual) | GAP-091-29 |
| `e2e_spec091_workspace_delete_zero_residue_1m` | IW5 | M-4.2 residue proof: 100-row CI / `EQ_SCALE_PROOF=1` 10k stand-in (true 1M residual) | GAP-091-30 |
| `e2e_spec091_cross_tenant_graph_leak` / `_ann_leak` | IW5 | LAW-I4: AGE + ANN cross-tenant denial on Postgres (incl. `LegacyNullAsWildcard`) | GAP-091-15 |

## Existing vs planned

Many rows above are **acceptance targets** (named for wave exit gates). Only the binaries listed under **CI wiring (exists today)** are present as test files in the working tree. Planned names without a matching `tests/*.rs` binary remain aspirational until implemented.

### Exists today (run these)

| Binary / suite | Package | Notes |
| --- | --- | --- |
| `e2e_spec091_wave_d` | edgequake-storage | Wave D drop / 42P01 / fence JOIN / write-stop |
| `e2e_spec091_console` | edgequake-storage | Advisor posture + 125 guard alignment |
| `e2e_spec091_job_control` | edgequake-storage | pause/resume/cancel ledger |
| `cli_migrate_console` | edgequake (bin) | CLI verbs + `dry-run` + `--confirm-drop` refuse |
| `e2e_spec091_queue_admission` | edgequake-tasks | QW admission / queued honesty |
| `e2e_spec091_queue_chaos` | edgequake-tasks | QW chaos |
| `contract_spec091_provider_budget` | edgequake-tasks | slot ledger |
| `contract_spec091_fairness_park_marker` | edgequake-tasks | park marker / wake |
| lib `contract_spec091_*` | edgequake-storage / edgequake-tasks | unit contracts |
| `e2e_document_deletion_postgres` | edgequake-api | wipe / schema-qualify (R-21/R-24) |
| `make spec091-upgrade-soak` | scripts | v0.22.0 GHCR → HEAD multi-tenant upgrade |
| `e2e_spec091_chunk_embeddings` | edgequake-storage | W3: `PgChunkEmbeddingIndex` port conformance (upsert/search/delete, idempotent, model-registry dedupe) |
| `e2e_spec091_vector_backfill` | edgequake-storage | W3: `eq_*_vectors` → `chunk_embeddings` engine backfill coverage + sampled equality + crash-resume |
| `e2e_spec091_recall_parity` | edgequake-storage | W3, M-3.1, R-05: typed recall@10 vs brute-force exact baseline |
| `e2e_spec091_vector_backend_dual` | edgequake-storage | W3: dual-write/read result-set parity + fallback counter on typed failure |
| `e2e_spec091_vector_retire` | edgequake-storage | W4: fleet verify → `VectorPosture.retirable` → guarded migration 126 (chunk-only drop) |
| `contract_spec091_boot_gate` | edgequake-api | B1: exit-78 refusal pins, downgrade protection, stale-flag shim, health agreement |
| `contract_spec091_strict_scope_headers` | edgequake-api | IW0: malformed/absent scope fail-closed |
| `contract_spec091_get_by_ids_typed` / `_unknown_family_loud` / `_llm_cache_scope` / `_shell_batch_write` | edgequake-storage | IW0/IW1 contracts |
| `e2e_spec091_ingestion_p95_budget` / `_retrieval_slo_protection` / `_hnsw_policy_converged` | edgequake-storage | IW1 Wave-0 scorecard + LD-06 |
| `e2e_spec091_fleet_recall_parity` / `contract_spec091_zero_runtime_ddl` | edgequake-storage | IW2 fleet cutover proofs |
| `contract_spec091_no_kv_facade` | edgequake-storage | IW3 census allowlist |
| `contract_spec091_capability_health` / `e2e_spec091_pg_matrix_smoke` | edgequake-api / storage | IW4 capability matrix + PG smoke |
| `proptest_spec091_key_grammar` | edgequake-storage | IW5 hermetic property tests |
| `chaos_spec091_crash_mid_batch` / `_lease_expiry_fencing` | edgequake-storage | IW5 chaos (nightly) |
| `e2e_spec091_cross_tenant_graph_leak` / `_ann_leak` / `_workspace_delete_zero_residue_1m` | edgequake-storage | IW5 isolation + residue |

### Planned (not yet as named binaries)

Remaining aspirational names: true process kill -9 chaos, true 1M residue soak, `contract_spec091_ports` expansion. **Descoped:** `proptest_spec091_chunk_writer` / `_cypher` (see IW table). Fleet-wide entity/rel/report coverage exists as `e2e_spec091_fleet_recall_parity` + migrations 130/131.

## CI wiring (exists today)

Local SSOT: `make spec091-gates` (mirrors `.github/workflows/spec091-data-layer.yml`).

```bash
make spec091-gates
# W3 chunk-embedding cutover (included in make target)
cargo test -p edgequake-storage --features postgres --test e2e_spec091_chunk_embeddings
cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_backfill
cargo test -p edgequake-storage --features postgres --test e2e_spec091_recall_parity
cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_backend_dual
# W4 + B1 + IW0–IW5 (also wired in .github/workflows/spec091-data-layer.yml)
cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_retire
cargo test -p edgequake-api --features postgres --test contract_spec091_boot_gate
cargo test -p edgequake-storage --features postgres --test e2e_spec091_fleet_recall_parity
cargo test -p edgequake-storage --features postgres --test contract_spec091_zero_runtime_ddl
cargo test -p edgequake-storage --features postgres --test e2e_spec091_hnsw_policy_converged
cargo test -p edgequake-storage --test proptest_spec091_key_grammar
cargo test -p edgequake-api --features postgres --test contract_spec091_capability_health
make spec091-upgrade-soak
# nightly: chaos_spec091_* + EQ_SCALE_PROOF=1 residue
```

## Doc 23 — Post-drop KV hot-path (KVH0–KVH2)

| Test | Proves |
| --- | --- |
| `contract_spec091_kv_ping_short_circuits_when_dropped` | Absent → ping no SQL |
| `e2e_spec091_health_no_kv_sql_post_drop` | Deep health: zero `eq_%_kv` statements |
| `e2e_spec091_hot_path_no_missing_kv_sql` | list/track/hydrate/wipe under drop: attempts == 0 |
| `contract_spec091_health_chunk_text_ssot_relational` | snapshot ≠ `"kv"` when authority relational |
| `contract_spec091_admission_stamps_track_id` | `documents.track_id` stamped |
| `contract_spec091_advisor_purge_aware_residue` | typed-backed keys → ReadyToDrop |
