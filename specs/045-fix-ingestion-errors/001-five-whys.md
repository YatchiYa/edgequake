# SPEC-045 — Five Whys (Post-Migration Ingestion Failure)

**Cross-ref:** [000-index](./000-index.md) · [003-code-is-law](./003-code-is-law.md)

---

## Incident statement

After migrating EdgeQuake from an earlier version into production, documents fail ingestion with statuses like `Failed`, `processing` (stuck), or disappear from the UI list while still present in storage.

---

## WHY 1 — Why do documents fail ingestion after migration?

**Because** the pipeline's persist phase returns `PipelineError::GraphError` or embedding/provider errors, and the task processor marks the document `failed` with `on_permanent_failure`.

**Evidence:** `ingestion_persister.rs` lines 343–357; `task_impl.rs` permanent failure path; `status_updates.rs` writes `failure_class`.

---

## WHY 2 — Why does the persist phase fail on an upgraded database?

**Because** one or more of these post-migration conditions is true:

1. **Graph merge batch errors** — `merge_entities_batch_global` or `merge_relationships_batch_global` returns `Err` (slow indexes, AGE version drift, entity type enforcement).
2. **Compensation failure** — merge partial-write rollback calls `delete_node` via parameterized Cypher; if binding is wrong, orphan nodes remain (SPEC-044).
3. **Vector dimension mismatch** — M042/M080 halfvec conversion incomplete; new embeddings don't match table schema.
4. **Embedding provider 400** — dense documents exceed token or input-count limits.
5. **LLM provider down** — Ollama/OpenAI unreachable after redeploy.

**Evidence:** `merger/mod.rs` 403–411, 461–465; `cypher_exec.rs`; `vector/migration.rs::ensure_dimension`.

---

## WHY 3 — Why do graph merges fail more often after upgrade than on fresh install?

**Because** upgraded volumes carry **historical graph scale and schema state** that fresh installs don't:

| Upgrade artifact | Effect on merge |
| ---------------- | --------------- |
| Large AGE graph without M038 `source_ids` GIN indexes | Slow `get_nodes_batch`; timeouts; batch failures |
| M043 AGE extension version jump | Cypher behavior changes; param contract stricter |
| Legacy un-normalized entity nodes (#217) | Strict entity type enforcement rejects writes |
| Missing M046 tenant isolation indexes | Scoped merge scans degrade |

Fresh installs run bootstrap reconcile inline on empty/small graphs. Production upgrades defer expensive index builds (M038 CONCURRENTLY) leaving **degraded readiness** or slow merge paths.

**Evidence:** `reconcile/m038.rs`; `entity_reconcile.rs`; SPEC-041 M078/M079 index repair.

---

## WHY 4 — Why doesn't migration bootstrap prevent all of these?

**Because** bootstrap is deliberately **split-brain safe** but not **ingestion-blocking safe** for all migrations:

- **Blocking readiness (correct):** M038 (missing indexes on large graph), M042 (pgvector < 0.8).
- **Non-blocking reconcile (risk):** M047 wsdoc backfill, M046 perf indexes, M040 entity CQRS backfill — ingestion proceeds even if reconcile is slow or partial.
- **sqlx marker-only pattern:** DDL lives in `support/*/apply.sql`; skipping ops scripts leaves markers applied but indexes missing.
- **No automatic re-ingest:** Failed documents stay `failed` until operator calls `reprocess`.

**Evidence:** `migration_bootstrap/mod.rs::is_ready_for_traffic`; `bootstrap-first-principles.md`.

---

## WHY 5 — Why is the system not bulletproof against these edge cases?

**Because** reliability mechanisms exist but have **gaps in classification, automation, and test gates**:

| Gap | Impact |
| --- | ------ |
| `classify_ingestion_failure` has no `GraphMerge` class | Operators get `unknown` + `retry` for merge failures |
| SPEC-011 EC-002 (429) still open | Transient rate limits become permanent failures |
| SPEC-038 timeout formula not wired to worker | Large PDFs hit 7200s cap |
| EC-003 silent 0-entity success | "Completed" docs with no knowledge |
| CI `continue-on-error` on AGE tests (SPEC-044 C-7) | Regressions can ship |
| No post-migration ingest smoke in release pipeline | Upgrade path untested at scale |

**Evidence:** `large_document_profile.rs` 41–100; SPEC-011 EDGE_CASES; `.github/workflows/postgres-integration.yml`.

---

## Root cause summary

```
Migration upgrade
    → historical graph scale + deferred indexes
        → merge batch slower / fails
            → compensation must rollback partial writes
                → (was) broken Cypher bind → permanent failed doc
                → (now) fixed bind but merge can still fail for index/scale reasons
                    → no auto-reprocess + weak failure_class
                        → operator-visible "ingestion broken"
```

**Primary root cause (production Graylog):** Graph merge error during persist on upgraded AGE graph.  
**Secondary root cause (amplifier):** Saga compensation Cypher regression (SPEC-044, **fixed in current source**).  
**Systemic root cause:** Migration bootstrap + ingestion pipeline lack unified **failure taxonomy + auto-recovery** for post-upgrade state.

---

## Corrective themes

1. **Classify** — Add `graph_merge` failure class with actionable `reprocess_full`.
2. **Gate** — Keep M038/M042 readiness; surface clear operator_action in `/health`.
3. **Reconcile** — M047/M046/M038 idempotent every boot (already wired).
4. **Recover** — Startup orphan recovery + `recover-stuck` + `reprocess` API (already wired).
5. **Automate** — Post-migration ingest smoke + SPEC-044 battle test as release gate.
6. **Harden** — Embedding 429 retry; permanent-400 skip; large-PDF routing (SPEC-038).
