# SPEC-091 — Simplify the Data Layer

> **Product pin (published):** v0.23.0 — schema through migration **141**; GHCR `ghcr.io/raphaelmansuy/edgequake:0.23.0`. Prior pin **v0.22.0** (`36c45b7`) stopped at migration **105**.
> **Code status (v0.23.0):** Waves A–D + W3/W4 chunks + **IW0–IW5** landed and **wire-closure verified** (`make spec091-gates` + `.github/workflows/spec091-data-layer.yml`). Typed SSOT for KV families; migrations **106–141** including **125** (KV drop), **126** (chunk-vector drop), **128–129** (listing indexes + chunk HNSW/ef=128), **130–131** (fleet embeddings + fleet drop, `--confirm-drop`), **132–137** (dim/outbox/RM), **138** (fairness hold), **139–141** (SPEC-098 spine/lifecycle). Boot migration gating (LD-15); fail-closed scope headers (IW0); Wave-0 scorecard binaries in CI; capability matrix on `/health`; typed vector serving **default**; proptest/chaos/cross-tenant leak gates. **v0.23.0 truth:** [16](16-post-cutover-assessment.md) · [18](18-full-completeness-assessment.md) · [19](19-improvement-plan.md) (WIRED + VERIFIED) · [20](20-ingestion-surface-assessment.md) (ingestion UI surfaces) · [21](21-ingestion-pipeline-data-model-improvement.md) (**IP0–IP2 landed**; IP3–IP5 open) · [22](22-ingestion-migration-system-assessment.md) (ingestion × migration reliability/perf/quality; **RM0–RM5**) · [24](24-worker-paradigm-improvement.md) (worker control plane kept; **WP0–WP1 landed**; WP2–WP5 open; **LD-18**). Residuals: KV facade census allowlist (shrinking); SPEC-120 descoped; fleet drop human-gated; true kill-9 / 1M soak; AGE index/AI citation contracts. Fence default **on** (escape `EDGEQUAKE_SERVING_FENCE=off`). Risks R-21..**R-30**; EC-28..EC-36.
> **Release status:** **Shipped in v0.23.0** (migrations **106–141**). Ops runbook: [docs/operations/spec091-upgrade-from-v0.22.0.md](../../docs/operations/spec091-upgrade-from-v0.22.0.md).
> **Residual realized risks (ops):** R-27 roll write-stop replicas before/with drop; R-28 fence JOIN → `public.chunk_serving_state`; R-29 typed mm-asset existence; R-30 confirmed-purge before presence-conservative abort. After 125, stale replicas fail ingest — roll all replicas current, then `POST /documents/reprocess` as needed (EC-36).
> **Input:** [00-raw-needs.md](00-raw-needs.md) (immutable expert study, fact-checked against the release tag). Pin-era “Today” in [02](02-first-principles.md)/[03](03-assessment.md) describes **v0.22.0**, not HEAD.
> **Inherits:** [SPEC-021 storage study](../021-storage-study/), [SPEC-058 data-layer hardening](../058-data-layer-hardening/), [SPEC-059 data-layer integrity](../059-data-layer-integrity/), [SPEC-088 data-layer](../088-data-layer/), [SPEC-089 health check](../089-health-check/), [SPEC-090 performance](../090-performance/)
> **Official-doc basis (July 2026):** PostgreSQL 18 ([release notes](https://www.postgresql.org/docs/18/release-18.html), [TOAST](https://www.postgresql.org/docs/18/storage-toast.html), [RLS](https://www.postgresql.org/docs/18/ddl-rowsecurity.html), [partitioning](https://www.postgresql.org/docs/18/ddl-partitioning.html), [generated columns](https://www.postgresql.org/docs/18/ddl-generated-columns.html)) · pgvector 0.8.x ([README](https://github.com/pgvector/pgvector): halfvec, iterative scan, multitenancy) · Apache AGE ([releases](https://github.com/apache/age/releases): PG18/v1.8.0-rc0 published 2026-07-09)

## Start here

1. [01-why.md](01-why.md) — WHY this refactor exists: one-paragraph problem, Five WHYs, causal chain (ASCII), cost of doing nothing.
2. [02-first-principles.md](02-first-principles.md) — the 4 axioms and 8 laws (`LAW-D1..D8`) every decision here derives from, each anchored to official docs and code evidence.
3. [03-assessment.md](03-assessment.md) — **pin v0.22.0** code assessment (path:line); corrections register for `00-raw-needs.md`. For HEAD after A–D, read [16](16-post-cutover-assessment.md).
4. [04-cross-ref-matrix.md](04-cross-ref-matrix.md) — master cross-reference: finding ↔ law ↔ code ↔ official doc ↔ measure ↔ test. The ID registry (SSOT for all identifiers).
5. [05-target-specification.md](05-target-specification.md) — target architecture: consolidated schema, domain ports (SOLID), serving-fence contract, SSOT map (partially achieved after A–D; W3 embeddings still target).
6. [06-implementation-plan.md](06-implementation-plan.md) — Waves 0–5 + A–D↔W mapping; KV-retirement DoD vs spec-complete residual.
7. [07-migration-engine.md](07-migration-engine.md) — migration engine + operator upgrade path (v0.22.0 → dry-run → confirm-drop → HEAD).
8. [08-performance-contract.md](08-performance-contract.md) — performance laws, workload budgets, release scorecard, acceptance numbers.
9. [09-risk-register.md](09-risk-register.md) — risks `R-01..R-30` with status (open / mitigated / residual-ops).
10. [10-edge-cases.md](10-edge-cases.md) — edge-case register `EC-01..EC-36` with mitigation and owning test.
11. [11-e2e-test-matrix.md](11-e2e-test-matrix.md) — `e2e_spec091_*` / `contract_spec091_*` / `chaos_spec091_*` tests mapped to waves and gates (exists vs planned).
12. [12-queue-admission-first-principles.md](12-queue-admission-first-principles.md) — queue & admission: WHY, axioms, `LAW-Q1..Q7`, capacity math (Little's Law), DRY/SOLID/SSOT mapping.
13. [13-queue-admission-target-spec.md](13-queue-admission-target-spec.md) — task state machine (code SSOT), provider-slot ledger DDL, admission resolver, API contract (queued + ETA).
14. [14-queue-admission-plan.md](14-queue-admission-plan.md) — waves QW0–QW3 with entry gates, mechanisms, exit gates, rollback.
15. [15-migration-console-cli.md](15-migration-console-cli.md) — CLI migration console (C0–C3 + `dry-run`); schema-derived advisor + gated guardrails.
16. [16-post-cutover-assessment.md](16-post-cutover-assessment.md) — **HEAD data-model & ops audit** after Waves A–D (SSOT map, law grades, residuals).
17. [17-boot-migration-gating.md](17-boot-migration-gating.md) — no silent auto-migrate at server start: `LAW-B1..B5`, refuse/exit-78 contract, per-env process, waves B0–B2 (LD-15).
18. [18-full-completeness-assessment.md](18-full-completeness-assessment.md) — six-criteria audit of the data layer at HEAD (0/6 fully met): evidence-cited grades, gap register `GAP-091-01..34`, falsifiable closure definitions.
19. [19-improvement-plan.md](19-improvement-plan.md) — closing program: `LAW-I1..I6`, waves **IW0–IW5** (safety+CI → perf baseline → vector fleet cutover → debt/flag retirement → PG16/17/18 best use → test hardening), `EC-I*`, `R-I*`, criterion-closure acceptance (LD-16, LD-17).
20. [20-ingestion-surface-assessment.md](20-ingestion-surface-assessment.md) — Active View · Document View · pipeline chrome; laws **LAW-IS1..IS4**; **IS0–IS1 landed** (`progress_counts` SSOT + single meter + per-type); IS2–IS3 open (queue ETA, phase copy + fence).
21. [21-ingestion-pipeline-data-model-improvement.md](21-ingestion-pipeline-data-model-improvement.md) — ingestion pipeline + typed data-model assessment; findings `F-IP-01..22`; laws **LAW-IP1..IP6**; waves **IP0–IP2 landed** (skip legacy upsert, batch ETA, batch CQRS, outbox + fence default on); IP3–IP5 deferred.
22. [22-ingestion-migration-system-assessment.md](22-ingestion-migration-system-assessment.md) — joint **ingestion × migration** audit (reliability / performance / quality) + data-model first principles (PG16/17/18 · AGE · O(n) · AI Eng July 2026); findings `F-RM-01..28`; laws **LAW-RM1..RM8**; waves **RM0–RM5** (outbox drain/soak → facade/wipe → AI write contract → AGE indexes → PG18 measure → release).
23. [23-post-drop-kv-hot-path.md](23-post-drop-kv-hot-path.md) — post-migration-125 KV hot-path closure: `KvRelationState` zero-SQL short-circuit, honest health SSOT, admission `track_id`, relational chunk counts, purge-aware advisor; laws **LAW-KVH1..KVH5**; waves **KVH0–KVH2**.
24. [24-worker-paradigm-improvement.md](24-worker-paradigm-improvement.md) — first-principles verdict on the **worker paradigm**: durable claim/lease control plane is correct; monolithic document `process()` is not; target = stage-bounded workers on Postgres (**LAW-WP1..WP8**, waves **WP0–WP5**, **LD-18**). Temporal deferred unless soak fails (LAW-WP8).

## Locked decisions

1. **LD-01 — Chunk text authority moves to `chunks.content`.** The generic KV chunk path is removed; the KV record becomes a temporary dual-write during Wave 1 only. (LAW-D6; F-091-02, F-091-10)
2. **LD-02 — One chunk identity: UUID everywhere.** The derived `{doc_id}-chunk-{n}` string is retired from storage; it survives only as a derivable read-fallback during Wave 1 dual-read. (LAW-D2; F-091-03)
3. **LD-03 — Runtime code never issues DDL.** `eq_*_kv`, `eq_*_vectors`, stats sidecars, triggers, and pattern indexes are replaced by migration-owned relations. Boot performs read-only verification plus job resumption. (LAW-D5; F-091-04, F-091-10)
4. **LD-04 — Apache AGE remains the traversal authority.** The relational `entities`/`relationships` read models stay optional CQRS projections. Graph work is not moved to recursive SQL. (F-091-09; divergence from SPEC-021 recorded in [03-assessment.md](03-assessment.md#divergence-from-spec-021))
5. **LD-05 — Portability lives at domain ports, not at a generic KV model.** Ports are batch-first, capability-declared, and defined by a conformance suite that runs against every adapter in CI. (LAW-D7; SOLID mapping in [05-target-specification.md](05-target-specification.md))
6. **LD-06 — One HNSW construction policy.** `ef_construction` currently exists as three values (32 in migration 071, 128 in `config.rs`, 64 in `docker/init.sql`). Wave 3 converges on a single policy resolved from one source, chosen by a recorded recall/size benchmark. Until then the runtime default (128) stands and no new 32-builds are created. (LAW-D5; F-091-14)
7. **LD-07 — Every behavioral change ships behind a runtime flag with dual reads and a logged fallback counter.** At most one irreversible operation ships per release; each destructive step follows at least one full soak after the cutover that made it safe.
8. **LD-08 — Data movement never runs at boot and never blocks readiness.** Long-running work is a descriptor executed by the migration engine with pause/resume/cancel and monotonic progress reporting. (LAW-D8)
9. **LD-09 — Serving visibility is fail-closed.** A chunk is query-visible only when text, embedding, graph linkage, and readiness state all agree. Where a deployment cannot commit atomically, the fence degrades visibility, never integrity. (LAW-D1, LAW-D3)
10. **LD-10 — Partitioning and quantization are measurement-gated.** They are adopted only after a reproduced threshold breach against the Wave 0 baseline, per pgvector's multitenancy guidance (list partitioning or separate tables). (LAW-D8)
11. **LD-11 — Provider in-flight budget is enforced cluster-wide via a Postgres slot ledger.** SKIP LOCKED acquisition + TTL + fencing, mirroring the task-claim pattern; no process-local-only semaphore may gate provider access (N replicas must not multiply provider load by N). `budget=0` short-circuits for cloud-only deployments. (LAW-Q3; F-091-18)
12. **LD-12 — Saturation is answered with an explicit queued state, never a silent hang or a 429.** Valid uploads beyond the soft bound are admitted (202) with `queue_position` + clamped EWMA ETA; task status transitions are defined in exactly one state-machine module (code is law). (LAW-Q2, LAW-Q4; F-091-17, F-091-19)
13. **LD-13 — Tenant fairness is weighted fair-sharing of the provider budget, not hard per-tenant caps.** Lanes carry DRR weights over the budget; a single active tenant receives the whole budget. (LAW-Q5; F-091-20)
14. **LD-14 — Operator guidance is schema-derived, not tribal knowledge.** The migration console derives "where am I / what next" live from the ledger, family flags, and the drop guard — it never persists a parallel copy and refuses any illegal or unsafe transition (gated guardrails, fail-closed). A `kv`/`dual` flag against a dropped store is surfaced and blocked, never silently honored. ([15-migration-console-cli.md](15-migration-console-cli.md); LAW-C1..C6)
15. **LD-15 — Server start never silently mutates versioned schema; schema apply is explicit, operator-gated, previewable.** `edgequake migrate` is the sole versioned-schema writer; serving boot verifies `_sqlx_migrations` and refuses (exit 78, actionable dry-run/migrate message) when the database is behind or newer than the binary. `EDGEQUAKE_ALLOW_BOOT_MIGRATE` is removed; `make dev` runs the migrate step visibly before the server. Additive idempotent object-ensure at boot is a separate, bounded class. ([17-boot-migration-gating.md](17-boot-migration-gating.md); LAW-B1..B5)
16. **LD-16 — Spec-complete DoD is the six-criterion closure table, CI-proven.** The six criteria (C1 dynamic-table retirement, C2 isolation, C3 debt removal, C4 benchmarked CRUD, C5 enforced tests, C6 PG16/17/18 best use) each carry a falsifiable closure definition whose proof is a CI-wired command ([18 §10](18-full-completeness-assessment.md)); closure claims without a green gate are void. This supersedes the "W3–W5 exit gates" phrasing of the earlier spec-complete DoD in [16 §8](16-post-cutover-assessment.md). ([19-improvement-plan.md](19-improvement-plan.md) §8; LAW-I1, LAW-I5)
17. **LD-17 — The vector cutover completes the fleet, not chunks only.** Entity/relationship/report vectors receive typed homes via migration-owned schema, dual-run, engine backfill+verify, and a guarded drop migration (127) that retires all `eq_*_vectors` relations and every vector runtime-DDL path; LD-03 then holds for the whole data layer. ([19-improvement-plan.md](19-improvement-plan.md) IW2; user decision 2026-07-30)
18. **LD-18 — Worker control plane retained; execution becomes stage-bounded on Postgres.** Ingestion keeps durable `SKIP LOCKED` claim workers, leases, and the provider-slot ledger. The unit of claim evolves from a monolithic document `process()` to **StageAttempt** classes (Prepare / Extract / Embed / Materialize / Lifecycle) under a user-visible **Job**, with provider slots held only during infer stages and specialized pools by bottleneck. External workflow engines (Temporal et al.) are out of scope unless WP soak acceptance fails (LAW-WP8). ([24-worker-paradigm-improvement.md](24-worker-paradigm-improvement.md); 2026-07-31)

## Surfaces

| Surface | Role |
| --- | --- |
| `edgequake/migrations/` | Sole schema owner after consolidation (LD-03) |
| `edgequake/crates/edgequake-pipeline/src/persistence/ingestion_persister.rs` | Single ingestion persistence point — the one place the relational chunk writer is added (DRY) |
| `edgequake/crates/edgequake-storage/src/traits/` | Current storage traits; domain ports land beside them before any wave exits |
| `edgequake/crates/edgequake-core/src/workspace_service_impl/workspace_ops.rs` | Live statistics defect (F-091-11) — fixed ahead of Wave 1 |
| `edgequake/crates/edgequake-api/src/state/migration_bootstrap/` | Boot reconcile hooks; extended to register/verify migration descriptors, never to run them |
| `docker/init.sql` (edgequake/docker) | Third `chunks` definition — retired by consolidation (F-091-13) |
| `GET /admin/migration-jobs` + `edgequake migrate status` CLI + `edgequake.migration_progress` SQL view | Progressive migration information (three surfaces, one ledger) |
| `edgequake migrate console` / `dry-run` CLI (advisor) | Derived operator guidance + preview-only dry-run + gated `--confirm-drop` ([15](15-migration-console-cli.md), [16](16-post-cutover-assessment.md)) |

## Verification

```bash
# Pin-era falsifiers (v0.22.0 @ 36c45b7 only — do NOT expect these on HEAD):
# grep -rn "INSERT INTO chunks" --include="*.rs" .        # pin: no matches (F-091-02)
# On HEAD: relational writer exists; see 16-post-cutover-assessment.md

# Workspace gates:
cargo test --workspace --lib
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# SPEC-091 wired data-layer gates (mirrors CI job spec091-data-layer):
# Requires DATABASE_URL with pgvector + AGE (make postgres-start).
make spec091-gates

# Individual suites (see 11-e2e-test-matrix.md):
cargo test -p edgequake-storage --features postgres --test e2e_spec091_wave_d
cargo test -p edgequake-storage --features postgres --test e2e_spec091_console
cargo test -p edgequake-storage --features postgres --test e2e_spec091_job_control
cargo test -p edgequake --features postgres --test cli_migrate_console
cargo test -p edgequake-tasks --test e2e_spec091_queue_admission
cargo test -p edgequake-tasks --test e2e_spec091_queue_chaos
cargo test -p edgequake-api --features postgres --test e2e_document_deletion_postgres
make spec091-upgrade-soak
```
