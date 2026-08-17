# 08 — Performance Contract

> Consolidation changes the performance problem: KV point reads cease to be a capacity variable; the dominant costs become embedding-index writes, HNSW residency, vacuum progress, graph projection throughput, and ingestion↔retrieval interference. This file converts performance from tuning flags into an **enforceable workload contract**. Every number is a gate, not an aspiration.
>
> **Progress after Waves A–D:** KV point-read capacity variable is removed **post-125** on HEAD. Placement rows that name `chunk_embeddings` as the embedding SSOT are still **W3 targets** (live path = `eq_*_vectors`). Fence/outbox atomicity (LAW-P4) is module-ready with fence default **off**. Scorecard gates for W3–W5 remain release/measurement-gated. See [16](16-post-cutover-assessment.md).

## Performance laws (LAW-P)

1. **LAW-P1 — Work that grows with total data stays off request paths** (LAW-D8): counts, relation discovery, index creation, reconciliation, fairness scans run asynchronously or from bounded summaries.
2. **LAW-P2 — Large immutable values stay away from frequently updated rows** (LAW-D6): text/embeddings immutable by generation; leases, progress, retry counters use narrow typed columns → HOT updates, less WAL, less TOAST churn.
3. **LAW-P3 — One vector belongs to one active ANN index**: global and per-workspace HNSW never index the same embedding generation concurrently (fixes F-091-07's double write amplification).
4. **LAW-P4 — Atomic visibility without unbounded transactions**: bounded TXs write chunks + outbox; the serving state is the user-visible fence (LAW-D3). PG18 async I/O accelerates the scan/vacuum-heavy verification phases ([release-18](https://www.postgresql.org/docs/18/release-18.html)).
5. **LAW-P5 — Every latency claim includes recall, scale, concurrency, cache state**: ANN latency without recall is incomplete evidence; a faster low-recall config must never appear as an improvement.

## Data placement by mutation pattern

| Data | Mutation | Target | Reason |
| --- | --- | --- | --- |
| Chunk text | immutable after ingestion | `chunks.content` (TOAST keeps spine scans narrow) | transactional authority; [TOAST](https://www.postgresql.org/docs/18/storage-toast.html) |
| Embedding | immutable per model generation | `chunk_embeddings` | model-scoped indexes; expand-and-contract re-embedding |
| Document lifecycle | low-frequency | `documents` | small indexed state row |
| Task lease/progress | high-frequency | narrow typed columns | HOT updates (changing columns absent from indexes) |
| Task request payload | immutable | separate JSONB column or child table | progress updates never rewrite the payload |
| Compensation history | append-heavy | `compensation_quarantine` | independent retention + retry indexes |

Default keeps content inside `chunks` (TOAST). A one-to-one `chunk_contents` table remains a **benchmark-driven** option when metadata-only scans, row updates, or buffer-cache pollution show a measured penalty. **Post-backfill caution (F-091-02 consequence):** every planner statistic, cache assumption, and size figure measured on today's *empty* `chunks` is invalid after W1 — `ANALYZE` + autovacuum-threshold review + buffer-headroom recompute + scorecard re-measurement are mandatory before any W3 comparison counts as evidence.

## HNSW capacity math (the primary capacity ratio)

`active HNSW bytes ÷ effective cache (shared_buffers + OS page cache)` — once the active graph stops fitting, random reads rise and tail latency destabilizes.

- 1,536-dim `halfvec` ≈ 3 KB raw before tuple + graph-link overhead → 10M vectors ≈ 30 GB payload alone; links, dead tuples, metadata, and duplicate indexes add more. Half-precision already halves this vs `vector` (shipped default, `capabilities.rs:83-92`).
- One table + partial expression indexes suits a **small number of active models**; list partitioning by `model_id` when generations multiply or predicate overhead is measurable; workspace partitioning only when filtered recall or operational isolation fails its gate (pgvector [multitenancy](https://github.com/pgvector/pgvector#multitenancy)).
- Each insert touches **one** active HNSW graph (LAW-P3): workspace promotion removes rows from the shared graph before the dedicated index becomes authoritative.
- `ef_construction` is one policy with one benchmarked value (LD-06): 32 (M071: −35% size, <2% recall loss) vs 128 (runtime default, SPEC-090 recall findings) — the W3 benchmark fixes a single number and records it in the schema-generation ledger.

## Bounded ingestion transactions (LAW-P4)

A document-spanning transaction pins the oldest xid, delays vacuum, accumulates WAL, inflates retry cost. Target: **250–1,000 chunks per TX** (rows + text + dedup + outbox), tuned against TX p95, WAL bytes/chunk, lock waits, vacuum delay; normal ingestion TX **< 2 s** under the declared hardware profile. FTS uses the stored generated tsvector; steady state maintains its GIN continuously; large imports may stage unindexed → build → attach **only with explicit delayed visibility**.

## Workload admission (pool budgets, initial)

| Pool | Budget | Policy |
| --- | --- | --- |
| Retrieval | 12–20 conns | short `statement_timeout`, read-biased, latency-protected |
| Ingestion | 4–8 | bounded TXs + bounded concurrency |
| Task claiming | 2–4 | very short TXs, `FOR UPDATE SKIP LOCKED` |
| Graph projection | 4–8 | explicit timeout, depth/frontier/node caps |
| Maintenance | 1–2 | migrations + concurrent index work only |

Every pool sets `statement_timeout`, `lock_timeout`, `idle_in_transaction_session_timeout`, `search_path`, `application_name`, `work_mem`; high `maintenance_work_mem` confined to maintenance. Ingestion admission responds to retrieval health: on retrieval-p95 SLO breach, reduce concurrent embedding upserts and pause optional index work. **One HNSW build per database** until measurements prove otherwise. Existing head-of-line guidance stands: pool size ≥ peak simultaneous holders; server `max_connections` ≥ sum of pools (`config.rs:85-89`).

## Bounded queues & bounded graphs

- **Task claims** bounded in *pending backlog*, independent of total history: fairness from a bounded candidate window; separate partial indexes for pending vs expired leases; two sargable `SKIP LOCKED` arms instead of a cross-status `OR`; terminal tasks monthly-partitioned or archived; keyset pagination on `(created_at, track_id)`; claims fetch no immutable payload; queue metrics from recent windows with explicit timeouts. Ladder: 100 / 10k / 100k / 1M pending — claim p95 + buffers touched bounded across the ladder.
- **Graph expansion** bounded by depth *and* frontier, visited nodes, returned edges, statement duration (depth-2 through hubs can exceed sparse depth-5). Native label-table queries use `= ANY($1::text[])` for stable plans; expression indexes match every cast; statistics refreshed after backfills and large ingestion waves; boot does read-only verification only.

## Scorecard (release-level evidence)

| Area | Required evidence (ID) |
| --- | --- |
| ANN search | p50/p95/p99, recall@10, recall@20, index bytes, cache state (M-0.1) |
| Filtered ANN | same metrics at 0.01% / 0.1% / 1% / 10% / 100% selectivity (M-0.2) |
| Ingestion | chunks/s, TX p95, WAL bytes/chunk, lock waits (M-0.3) |
| Full-text | p95, GIN pending-list behavior, buffers touched (M-0.4) |
| Graph expansion | p95 by depth, frontier, visited nodes, edges (M-0.5) |
| Task queue | claim p95 + buffers by backlog depth (M-0.6) |
| Vacuum | oldest xid age, dead tuples, vacuum duration, blocked cleanup (M-0.7) |
| Connection pools | acquire wait, active, timeouts, utilization by workload (M-0.8) |
| Deletion | duration + residual artifacts at 1k / 100k / 1M chunks (M-0.9) |
| Re-embedding | duration, inference cost, coverage, peak dual-generation storage (M-0.10) |
| Chunk backfill | rows/s, TX p95, WAL bytes, relation+TOAST growth, autovacuum progress, replica lag, retrieval p95 during run (M-0.11) |

Method: every representative query records `EXPLAIN (ANALYZE, BUFFERS, VERBOSE, SETTINGS)`; ANN compared with exact search by disabling index scans in a TX; results record PostgreSQL/pgvector versions, hardware, dataset shape, concurrency, warm/cold cache, index definitions.

## Acceptance numbers (per wave; SSOT for M-x.y in [04](04-cross-ref-matrix.md))

| Wave | Metric | Target |
| --- | --- | --- |
| W0 | Scorecard coverage | every metric recorded with hardware/dataset/concurrency/cache metadata |
| W1 | Query-visible chunks lacking relational text/embedding/readiness | **0** — visibility fail-closed |
| W1 | Backfill coverage of live KV chunk keys | **100%**, measured before any checksum gate is trusted |
| W1 | Checksum disagreement after full coverage | **0** unexplained for one complete ingestion cycle |
| W1 | Ingestion TX p95 + retrieval p95 during/after backfill | no regression vs W0 baseline; backfill throttles first |
| W1 | Storage-specific imports in app modules | **0** (CI dependency lint) |
| W1 | Conformance suite, every registered adapter | green, including in-memory |
| W2 | Vector tables outside current schema generation | **0** after resumable convergence job |
| W2 | Legacy full-precision `vector` columns | **0**, recall@10 verified vs pre-conversion baseline |
| W2 | Rows or readers in `eq_*_kv` | **0** before any drop |
| W3 | Recall@10 parity per migrated relation | within declared gate vs exact-search baseline (M-3.1) |
| W4 | Quarantine oldest age | < 15 min during normal operation (M-4.1) |
| W4 | Residual artifacts after workspace deletion | **0** across relational/vector/AGE (M-4.2) |
| W4 | Normal ingestion TX p95 | < 2 s, no vacuum-blocking long TX |
| W4 | Retrieval pool acquisition p95 during sustained ingestion | < 10 ms, zero starvation events |
| W5 | HNSW membership per embedding generation | exactly 1 active ANN index (M-5.1) |
| W5 | Task claim p95, 100 → 1M pending | bounded by SLO, no linear backlog growth |
| W5 | Recall@10/@20 across filtered ladders | no regression beyond declared gate at latency target |
