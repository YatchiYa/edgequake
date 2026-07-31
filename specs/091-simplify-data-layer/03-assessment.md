# 03 — Assessment: current code vs July 2026 practice

> Every claim below cites the code that proves it (path:line, verified at pin `36c45b7` / branch `62e6adb`). Findings carry IDs `F-091-XX` mapped to the raw study's `S-XX` in [04-cross-ref-matrix.md](04-cross-ref-matrix.md). Corrections to `00-raw-needs.md` are collected in the [Corrections register](#corrections-register) — the raw study stays immutable.
>
> **As-of banner:** this document is the **v0.22.0 pin assessment**. Claims such as “Nothing writes `chunks`” / “Chunk text SSOT is KV” are **true at the pin** and **false as a description of HEAD** after Waves A–D. For working-tree truth (SSOT map, ops, law grades, residuals), read [16-post-cutover-assessment.md](16-post-cutover-assessment.md).

## Evidence hierarchy

| Class            | Anchor                                                                 | Use                                       |
| ------------------| ------------------------------------------------------------------------| -------------------------------------------|
| Released product | v0.22.0 @ `36c45b7`                                                    | Authoritative behavior                    |
| Default branch   | `62e6adb` (4 doc/test commits later)                                   | Drift detection — no semantic drift found |
| Target practice  | PostgreSQL 18, pgvector 0.8.x, Apache AGE PG18/v1.8.0-rc0 (2026-07-09) | Recommendation or benchmark gate only     |

## Verified write-path facts (the load-bearing claims)

1. **Nothing writes `chunks`.** Repo-wide search for `INSERT INTO chunks` / `INTO chunks` returns zero SQL writers. The only `FROM chunks` in Rust is the workspace-stats query (`edgequake/crates/edgequake-core/src/workspace_service_impl/workspace_ops.rs:448,451`). (F-091-02, F-091-11)
2. **Chunk text SSOT is KV.** `build_chunk_kv_records` → `kv.upsert` (`edgequake/crates/edgequake-pipeline/src/chunk_storage.rs:12-22`; persisted via `edgequake/crates/edgequake-pipeline/src/persistence/ingestion_persister.rs:292,298`); the module header declares "chunk text SSOT in KV" (`chunk_storage.rs:1`).
3. **Vector rows carry `content_ref`, never inline content** — contract-tested (`chunk_storage.rs:63-108`, test at 187-210). Writer: `INSERT INTO {table} ... FROM unnest(...)` (`edgequake/crates/edgequake-storage/src/adapters/postgres/vector/storage_impl.rs:225`).
4. **Two chunk identities coexist.** Relational: `chunks.id UUID DEFAULT gen_random_uuid()` (`edgequake/migrations/001_init_database.sql:205-219`, incl. `UNIQUE(document_id, chunk_index)`). Operational: `{doc_id}-chunk-{index}` (`edgequake/crates/edgequake-storage/src/kv_key_schema.rs:66`). Vector PK: `id TEXT` (`adapters/postgres/vector/ddl.rs:206-216`). (F-091-03)
5. **Runtime DDL with discarded errors.** `eq_*_vectors` + six `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` executed with `.ok()` (`vector/ddl.rs:267-285`); `eq_*_kv` + two pattern indexes at runtime (`adapters/postgres/kv.rs:107-155`). (F-091-04, F-091-10)
6. **Stats defect is live.** Both `chunk_count` and `embedding_count` are the identical subquery `(SELECT COUNT(*) FROM chunks WHERE workspace_id = $1)` (`workspace_ops.rs:448,451`) → every workspace reports 0 and 0. (F-091-11)
7. **`chunks` has three competing definitions.** Migration 001 (FK-constrained), migration 002 (`edgequake/migrations/002_add_tasks_table.sql:51`), and `edgequake/docker/init.sql:96-119` — the last with **relaxed FKs** (`tenant_id UUID`, `workspace_id UUID`, no `REFERENCES`), plus an HNSW index on the never-written `embedding` column at `ef_construction=64` (`init.sql:562-563`), surfaced through view `edgequake.chunks`. (F-091-13)
8. **Half precision is already the default.** `VectorStorageMode::from_env()` → `Half` when unset (`adapters/postgres/capabilities.rs:83-92`), pinned by unit test. Residual: `HnswRuntimePolicy::default()` still hardcodes `Full` (`hnsw_runtime_policy.rs:25-40`) — struct default disagrees with env default. (F-091-06)
9. **pgvector floor enforced.** `PGVECTOR_MIN_CVE_SAFE = "0.8.2"` (`capabilities.rs:37`); dimension policy `HNSW_MAX_DIM_VECTOR=2000 / HALFVEC=4000` (29-30); `AnnIndexPolicy::resolve` promotes (2000,4000] to halfvec (56-79). Matches pgvector's documented ceilings (vector 2,000 / halfvec 4,000).
10. **Filtered ANN is a strength.** `hnsw.iterative_scan = relaxed_order`, `max_scan_tuples = 20000`, `ef_search = clamp(4·K, 40, 1000)` (`vector/search_tuning.rs:103-122`), exact reorder via materialized CTE + `ORDER BY distance + 0` (`ann_exact_reorder_policy.rs:105-121`) — exactly pgvector's documented relaxed-order + materialized-CTE pattern.
11. **Statement-level counters are current practice.** `FOR EACH STATEMENT` + transition tables (`adapters/postgres/row_count_stats.rs:159-213`). The row-lock concern applies only to stale pre-self-heal schemas. (F-091-08 narrowed)
12. **Legacy 8-hex slugs persist by design.** `workspace_slug_legacy` = `workspace_id.to_string()[..8]` (`traits/workspace_vector.rs:105-107`), superseded by full-UUID slugs (100-102) with runtime resolution preferring existing tables (`adapters/postgres/workspace_table.rs:8-77`). Collision risk (S-12) is retired for **new** tables; legacy tables have **no retirement ledger**. (F-091-12, F-091-16)

## Corrections register

| # | 00-raw-needs says | Code truth (verified) | Disposition |
| --- | --- | --- | --- |
| C-1 | `chunk_storage.rs` is in edgequake-storage | `edgequake/crates/edgequake-pipeline/src/chunk_storage.rs` | Corrected; F-091-15 sharpened (writer/reader split across pipeline/core/api crates) |
| C-2 | `compensation.rs` under `adapters/postgres/` | `edgequake/crates/edgequake-storage/src/compensation.rs` (crate root) | Corrected |
| C-3 | Stats defect in edgequake-storage `workspace_ops.rs` | `edgequake/crates/edgequake-core/src/workspace_service_impl/workspace_ops.rs:444-454` | Corrected |
| C-4 | Traits live in edgequake-core | `edgequake/crates/edgequake-storage/src/traits/` (`kv.rs:53`, `vector.rs:236`, `graph.rs:180`) | Corrected; matters for where ports land (LD-05) |
| C-5 | HNSW `ef_construction=32` globally (migration 071), graded "aggressive" | 071 pins **32** (`migrations/071_hnsw_optimize.sql:133-138`, deliberate: −35% index size, <2% recall loss); runtime default is **128** (`config.rs:96-114`, SPEC-090 F-090-24/25); `init.sql` builds a third index at **64** | Upgraded to finding **F-091-14**: one knob, three values, three schema sources — the standing proof that runtime DDL defeats SSOT |
| C-6 | S-12 (8-hex identifiers) unmitigated | Mitigated for new tables by full-UUID slugs (F-090-17); legacy tables persist with no retirement mechanism | F-091-12 narrowed; new F-091-16 (retirement ledger) |
| C-7 | AGE evaluated "per image" at 1.7 | AGE **PG18/v1.8.0-rc0** published 2026-07-09 with `pg_upgrade` support functions ([releases](https://github.com/apache/age/releases)) | Upgrade-path input to [07-migration-engine.md](07-migration-engine.md) |
| C-8 | Raw study Table 1.2 row "AN index parameters — Amber" | With C-5, the Amber belongs to **knob triplication**, not to any single value | Re-scored below |

## Grading against July 2026 practice (13 + 2 dimensions)

| # | Dimension | Current (evidence) | July 2026 best practice | Grade |
| --- | --- | --- | --- | --- |
| 1 | Vector element type | `halfvec` default, env-pinned, dimension-promotion policy (facts 8, 9) | Half precision by default + dimension ceiling | **Green** |
| 2 | Distance metric | Cosine-only, documented + tested (`capabilities.rs:15-19`) | Deliberate, enforced metric | **Green** |
| 3 | Extension version safety | pgvector ≥0.8.2 floor, prefer 0.8.5; AGE capability gates (`capabilities.rs:33-42`) | Capability probing tied to security floors | **Green** |
| 4 | ANN index parameters | m=16 everywhere; **ef_construction = 32 (migration) / 128 (runtime) / 64 (init.sql)** (C-5) | One policy, one source, tuned per table | **Red → F-091-14** |
| 5 | Filtered ANN | relaxed_order + bounded scan + ef scaling + exact reorder (fact 10) | Adaptive iterative scans with exact final ordering | **Green** |
| 6 | Quantization | None (by design) | Adopt only after memory pressure + recall benchmark | **Conditional** (Wave 5 gate) |
| 7 | Index build safety | CIC for non-empty, INVALID detection, bounded `lock_timeout`/`maintenance_work_mem` (`vector/ddl.rs:80-160`) | Exactly this | **Green** |
| 8 | Multi-tenancy | Forced RLS (migration 096), per-workspace tables + denormalized columns | RLS + partitioning by tenant when recall/ops require; pgvector: "list partitioning or separate tables" | **Amber** (Wave 5 gate) |
| 9 | Chunk text authority | KV JSONB; `chunks.content NOT NULL` unpopulated; writable `content_tsv` backfilled via cross-store lookup (migration 091) | One authoritative row, FK-reachable, generated tsvector over the same value | **Red → F-091-02** |
| 10 | Hybrid search | GIN over `content_tsv` + HNSW | RRF over both, one round trip | **Amber** (Wave 4) |
| 11 | Graph layer | AGE traversal authority + optional CQRS read model, native helpers + reconciled indexes (067/075/083/086) | Exactly this, drift measured | **Green** (keep; LD-04) |
| 12 | Referential integrity | No FK possible: vector relations are runtime-created; identity types differ (facts 4, 5) | Presence enforced by FK + readiness contract | **Red → F-091-01, F-091-03** |
| 13 | Lifecycle counters | Writer-maintained `documents.chunk_count`; statement-level `eq_*_stats`; live `COUNT(*)` over empty `chunks` | Projections of a state machine, labeled as such | **Red → F-091-08, F-091-11** |
| 14 | Schema lifecycle | Runtime DDL, discarded errors, 3 definitions of one relation, no generation marker | Numbered digest-verified migrations only (LAW-D5) | **Red → F-091-04, F-091-13** |
| 15 | AI-engineering readiness | No serving fence, no re-embedding expand-and-contract (dimension change can discard vectors), quarantine without drainer | Readiness gate before serving; model-generation expand/contract; DLQ with SLOs | **Red → F-091-01, F-091-05** |

**Score: 6 Green · 2 Amber · 1 Conditional · 6 Red.** The vector engine is at or above mid-2026 practice. Every red is a *contract* failure (identity, authority, ownership, lifecycle), not a tuning failure — which is why the plan orders contract repair before any physical optimization.

## Divergence from SPEC-021

Recorded, not glossed: SPEC-021 recommended repairing the current topology in place to preserve Cypher. LD-04 preserves AGE traversal, removing that objection; SPEC-021's five unenforced invariants become enforceable via FK + fence + migration ownership. Agreement on the problem, deliberate choice of database-enforced over convention-enforced invariants. See [02-first-principles.md](02-first-principles.md#the-one-disagreement-this-spec-settles-by-law-not-by-vote).

## Falsification queries (run before Wave 0)

```sql
-- 1. Spine population (expect 0 rows)
SELECT count(*) FROM chunks;
SELECT relname, n_live_tup, n_tup_ins FROM pg_stat_user_tables
 WHERE relname IN ('chunks','documents','entities','relationships');

-- 2. Backfill scale implied by KV
SELECT count(*) AS kv_chunk_keys FROM eq_eq_default_kv WHERE key LIKE '%-chunk-%';

-- 3. Stats defect (expect identical zeros)
SELECT (SELECT count(*) FROM chunks WHERE workspace_id = $1) AS reported_chunks,
       (SELECT count(*) FROM chunks WHERE workspace_id = $1) AS reported_embeddings;

-- 4. Runtime relation fleet + generation drift
SELECT tablename FROM pg_tables
 WHERE schemaname='public' AND (tablename LIKE 'eq\_%\_kv' OR tablename LIKE 'eq\_%\_vectors') ESCAPE '\';

-- 5. HNSW ef_construction fleet check (expect a mix of 32/64/128 — F-091-14)
SELECT indexname, indexdef FROM pg_indexes
 WHERE schemaname='public' AND indexdef ILIKE '%hnsw%';
```

If query 1 returns rows or lifetime inserts, F-091-02 is refuted and this spec must be revised before proceeding — population claims are cheap to falsify and expensive to be wrong about.
