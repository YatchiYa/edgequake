# 06 — Implementation Plan (Waves 0–5)

> Ordered by dependency and reversibility, never by expected value. Every wave states: **entry gate → mechanism → exit gate → rollback**. A wave that fails its evidence twice returns to its previous state (execution rule 4).
>
> **Release reality (HEAD):** Waves A–D executed the **KV-retirement** slice of W0–W2 as **one unreleased train** (migrations **106–125**). Product pin remains **v0.22.0** (≤105). Next tagged release ships 106–125; **W3+** stay on later tags. Do not treat the original v0.22.1 / v0.23.x / v0.24.x train as the live path — see [16-post-cutover-assessment.md](16-post-cutover-assessment.md).
>
> **Wave taxonomy (A–D ↔ W0–W2):** A ≈ W0/W1 foundation (engine + typed tables + chunk authority) · B ≈ W1/W2 family cutovers + SQL backfills · C ≈ write-stop + console · D ≈ migration 125 contract. **W3 chunk-embedding cutover is now implemented (chunks only, flag-gated); W3 entity/rel/report vectors, W4, W5 remain open.**

## Sequencing invariants (why this order is the only safe order)

1. **Measurement precedes migration** — every later gate compares against the Wave-0 baseline.
2. **Text authority precedes KV removal** — the generic store cannot die while it is the declared SSOT.
3. **KV removal precedes embedding consolidation** — migrating vectors first would carry `content_ref` semantics and string identity into the new relation.
4. **Enforceable identity precedes the serving fence** — a fence over unenforceable references is advisory.
5. **Physical layout last** — partitioning/quantization thresholds are observable only after the write path stabilizes.
6. **Interface boundaries precede storage change** — each wave moves rows behind a domain port that already passes its conformance suite.

```ascii
        ┌─────────┐    ┌───────────────┐    ┌──────────────┐    ┌───────────────┐
        │ W0      │───▶│ W1            │───▶│ W2           │───▶│ W3            │
        │ baseline│    │ text authority│    │ remove KV    │    │ typed         │
        │ + schema│    │ writer+backfill│   │ family by    │    │ embeddings +  │
        │ cleanup │    │ dual-read     │    │ family       │    │ UUID FKs      │
        └─────────┘    └───────────────┘    └──────────────┘    └───────┬───────┘
              ▲                                                         │
              │              ┌───────────────┐    ┌──────────────┐      ▼
              │              │ W5            │◀───│ W4           │   ┌──────────────┐
              └──────────────│ measured      │    │ serving      │   │ (FKs enforced│
               thresholds    │ scaling:      │    │ lifecycle:   │   │  here)       │
               re-measured   │ partition +   │    │ outbox+fence │   └──────────────┘
                             │ quantization  │    │ +DLQ drain   │
                             └───────────────┘    └──────────────┘
```

## Pre-W1 patch (ships with v0.22.1, independent of waves)

**Fix F-091-11 and F-091-06 first** — they are small, live defects that must not be entangled with migration row-count changes:

- `workspace_ops.rs:448,451` — stop counting the empty `chunks` table twice. Interim truth: `chunk_count` ← KV chunk-key count per workspace (`count_embedded_chunks_for_docs` or documents rollup), `embedding_count` ← vector stats sidecar sum per workspace; both labeled as projections (LAW-D4). After W1 cutover, both re-point to `chunks` / `chunk_serving_state`.
- `hnsw_runtime_policy.rs:25-40` — align `HnswRuntimePolicy::default()` with `VectorStorageMode::from_env()` (Half).
- Freeze F-091-14: no new index builds may use migration-071's 32; document 128 as the interim single value.

## Wave 0 — Baseline & schema consolidation (no behavior change)

**Entry:** none.
**Mechanism:**
1. Record the full scorecard ([08 — Performance Contract](08-performance-contract.md)) with hardware, dataset shape, concurrency, cache state.
2. Inventory every `eq_%_kv` / `eq_%_vectors` relation: columns, indexdefs (capture each `ef_construction`, F-091-14), row counts, legacy vs full slugs (F-091-12/16).
3. Consolidate `chunks` to one definition (F-091-13): migrations become the sole narrator; `docker/init.sql` consumes or generates from it; view `edgequake.chunks` recreated from the single definition.
4. Run falsification queries ([03 — Assessment](03-assessment.md)); record `chunks` row count.
5. Define domain ports + conformance suite skeleton (LD-05); CI dependency lint stub.
**Exit gate:** scorecard complete with environment metadata; single `chunks` definition deployed; ports compile with in-memory adapter green.
**Rollback:** not applicable (no data change).

## Wave 1 — Relational text authority (a build, not a verification)

**Entry:** W0 exit; ports + green suite; single `chunks` definition.
**Mechanism (three ordered steps):**
1. **Writer** — `DefaultIngestionPersister` gains a relational chunk insert (`id, document_id, workspace_id, chunk_index, content` + spans) inside the *same bounded transaction* as dedup state; `UNIQUE(document_id, chunk_index)` makes it idempotent under retry (F-091-15: exactly one writer site, used by both core and API paths). KV chunk keys continue (dual-write).
2. **Backfill** — a migration-engine descriptor ([07](07-migration-engine.md)): keyset cursor over `(document_id, chunk_index)`, batches 250–1,000, `INSERT ... ON CONFLICT DO NOTHING`, maintenance pool, throttled against retrieval p95 and replica lag. GIN/`content_tsv` created after bulk load above ~1M rows; continuously maintained below.
3. **Verify** — publish four separate metrics: coverage %, checksum mismatches, missing keys, missing rows (different causes, different responses). `ANALYZE chunks` + join partners; re-measure scorecard subset.
**Flag:** `EDGEQUAKE_CHUNK_TEXT_AUTHORITY = kv | dual | relational` (read path moves with logged KV fallback counter).
**Exit gate:** coverage = 100% of live KV chunk keys (M-1.1); zero checksum mismatches for one full ingestion cycle (M-1.2); zero fallback reads for one release soak (M-1.3); ingestion p95 inside W0 budget (M-1.4); zero storage-specific imports in app modules (lint).
**Rollback:** flag flip to `kv` — writers still emit KV keys throughout the wave. Free.

## Wave 2 — Remove the generic KV store (family by family)

**Entry:** W1 exit; no writer emits chunk keys.
**Mechanism:** per key family (Table in [05](05-target-specification.md#typed-replacements-for-kv-key-families)), one family per change so failure isolates: `wsdoc:` → `documents.workspace_id`; `staging:hash:` → `ingestion_dedup`; `compensation_quarantine:` → typed DLQ (schema in W4, landed here as table + drain); metadata → `documents`. Reads keep a compatibility path for one release.
**Flag:** per-family `EDGEQUAKE_KV_FAMILY_<NAME> = kv | relational`.
**Exit gate:** zero readers and zero rows in every `eq_*_kv` (M-2.1); then drop relations, stats sidecars, trigger functions, pattern indexes, and the runtime initialization code (F-091-10).
**Rollback:** flag flip before the drop; restore after. **The drop ships alone** — the wave's single irreversible step, one release after zero-reader evidence (LD-07).

## Wave 3 — Typed embeddings & identity convergence

**Entry:** W2 exit (no migrated row depends on string identity).
**Mechanism:** create `embedding_models` + `chunk_embeddings`; migrate rows per source relation via engine descriptor keyed `(model_id, chunk_id)` with typed routing columns; build model-scoped partial HNSW concurrently, one build per database; validate recall/latency vs exact search on the same data *per relation* before retiring it; schema-generation ledger records per-relation progress (F-091-16, resumable); legacy 8-hex slug relations retired through the same ledger (F-091-12). **Expand-and-contract** replaces dimension-change vector loss: new model generation inserts alongside; old generation retired after re-embedding coverage gate. F-091-14 resolved here by benchmark: one `ef_construction` policy, one value, recorded in the ledger.
**Flag:** `EDGEQUAKE_VECTOR_BACKEND = legacy_tables | chunk_embeddings`.
**Exit gate:** recall@10 parity per migrated relation (M-3.1); FKs enforced; vector tables outside current schema generation = 0; legacy full-precision `vector` columns = 0 (with recall check vs pre-conversion baseline).
**Rollback:** read redirect to retained source relation (free until its retirement gate passes).

> **Status (HEAD, chunks only):** the chunk-vector slice of W3 is implemented and wired — `VectorBackend` flag SSOT (fail-safe default `legacy_tables`), `PgChunkEmbeddingIndex` behind the `EmbeddingIndex` port, ingest **dual-write** (legacy + typed), query **dual-read** with a logged fallback counter (`vector_backend_fallback_total`), engine job `w3-chunk-embedding-backfill` + `verify_chunk_embedding_backfill`, and a console **VECTOR** posture row (backend, job state, verify, row counts). E2e: `e2e_spec091_chunk_embeddings`, `e2e_spec091_vector_backfill`, `e2e_spec091_recall_parity`, `e2e_spec091_vector_backend_dual`. **Default remains `legacy_tables`** — flipping to `chunk_embeddings` is operator-gated on the recall-parity gate. **Out of scope this turn:** entity/relationship/community-report vectors, `eq_*_vectors` drop, runtime vector-DDL retirement.

## Wave 4 — Serving lifecycle hardening

**Entry:** FKs enforced (W3) — the fence is otherwise advisory (invariant 4).
**Mechanism:** chunk + text + dedup + outbox event commit in one bounded TX (LAW-D3); embedding and graph workers apply mutations idempotently and advance `chunk_serving_state`; query path filters `state='ready'` (fail-closed, LD-09); compensation quarantine becomes the typed DLQ with bounded retry, backlog-age and terminal-failure SLOs (M-4.1); workspace deletion reports completion only after relational cascade **and** projection absence are both verified (M-4.2); hybrid retrieval collapses to one engine round trip with RRF (assessment #10).
**Flag:** `EDGEQUAKE_SERVING_FENCE = off | on`.
**Exit gate:** zero query-visible chunks lacking text/embedding/readiness; zero deletion residue at 1M chunks (M-4.2); quarantine oldest age < 15 min (M-4.1).
**Rollback:** disable fence → unfenced visibility, no data loss. Free.

## Wave 5 — Measured scaling (evidence-gated only)

**Entry:** stable write path + **reproduced threshold breach** vs W0 baseline (LD-10).
**Mechanism (each independently gated):** list partitioning by `model_id` or workspace when filtered recall/vacuum/size/isolation crosses a recorded limit (pgvector multitenancy guidance); binary quantization gated by HNSW memory residency + recall benchmark, reusing the existing materialized exact-reorder path for reranking; hot-workspace policy retune (F-091-07: exactly one active HNSW membership per generation, M-5.1; promotion threshold re-derived from churn data).
**Exit gate:** improvement vs W0 baseline with no recall regression beyond the declared gate.
**Rollback:** detach partition / drop index (recoverable); a completed partition split is not — strategy decided once, documented.

## Execution rules (hold across all waves)

1. Every behavioral change behind a runtime flag with dual reads + logged fallback counter (LD-07).
2. No destructive step in the same release as the cutover that made it safe (≥1 soak).
3. At most one irreversible operation per release (drops, type conversions, partition attachments).
4. An exit gate failing twice returns the wave to its previous state — no documented exceptions.
5. No storage-specific type in an application module; conformance suite green on every adapter before any wave exits.

## Permitted parallelism

- W0 instrumentation continues permanently (it becomes the scorecard harness).
- W2 may process independent key families concurrently (different facts).
- W3 inventory/index preparation may run during W2, writing nothing until W2 exits.
- W5 investigations may run in staging at any time (evidence, not production change).

## Definition of Done — KV retirement (Waves A–D / W0–W2 contract)

These boxes mean **in-tree KV retirement is release-ready**, not that SPEC-091 is complete.

- [x] F-091-01..16 infrastructure for KV retirement landed (schema + ports + flags + tests); full matrix e2e remains partially aspirational ([11](11-e2e-test-matrix.md))
- [x] Relational chunk writer exists at one site (`ingestion_persister` + `PostgresChunkRepository`); `chunks` consolidated in init.sql
- [x] Boot remains read-only for data movement; migration mode + progress API (`GET /admin/migration-jobs`)
- [x] Workspace stats Pre-W1 truthful projections (F-091-11); post-drop HEAD uses relational chunk counts
- [x] Serving fence module + migration 109 (flag default off)
- [x] Migrations **106–125** checksum-locked; scale gates require evidence (W5)
- [x] Scorecard module + contract tests green (subset)
- [x] Upgrade soak GREEN: `make spec091-upgrade-soak` (dry-run preview + `--confirm-drop` + post-drop list/isolation/wipe/assets); runbook [docs/operations/spec091-upgrade-from-v0.22.0.md](../../docs/operations/spec091-upgrade-from-v0.22.0.md). Rehearse on a restored production dump before tagging.

## Definition of Done — spec-complete (still open)

- [x] W3 **chunk** typed embeddings cutover implemented behind `EDGEQUAKE_VECTOR_BACKEND` (dual-write/read + backfill + verify + console posture + recall-parity e2e); default still `legacy_tables` pending parity flip. Entity/rel/report vectors + runtime-DDL retirement remain open.
- [ ] W4 1M-chunk delete residue / mid-migration delete e2e; fence-on query proof as default-capable
- [ ] W5 partitioning/quantization — evidence-gated (not stubs)
- [ ] Product tag ships migrations 106–125 (pin leaves v0.22.0)
- SPEC-120 first-class operations API — separate programme; orphaned WIP (see `specs/92-task-system/`)

**Residual after Waves A–D:** see [16-post-cutover-assessment.md](16-post-cutover-assessment.md) §§6–8.
