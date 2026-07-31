---
title: "PG18 capability-gated adoption notes (SPEC-091 IW4)"
---

# PG18 capability-gated adoption notes (SPEC-091 IW4)

EdgeQuake supports PG16, PG17, and PG18 with **one SQL path**. PG18-only features are exposed via runtime probes (`PostgresCapabilityProbe` / `/health.schema.postgres_capabilities`) and adopted only when they do not break PG16.

## Adopted via capability probe

| Feature | Probe / signal | Behavior |
| --- | --- | --- |
| **uuidv7 document IDs** | `uuidv7_available` (`capabilities.rs`) | PG18 + `uuidv7()` present → `DocumentIdGenerator::UuidV7`; else uuidv4 |
| **Iterative ANN scans** | `iterative_scan_available` (pgvector ≥ 0.8.0) | Filtered queries set `hnsw.iterative_scan` + `max_scan_tuples` |
| **AGE jsonb↔agtype casts** | `age_jsonb_agtype_cast_available` (AGE ≥ 1.8.0-rc0) | Probe only on `/health`; app SQL stays portable until measured win (SPEC-091 RM3) |

## Documented deferrals (interim fixes in place)

### Virtual generated column for workspace metadata (GAP-091-24)

**Deferred.** PG18 supports virtual generated columns that could index `metadata->>'workspace_id'` without duplicating storage. A version-sensitive migration would break PG16 fleets.

**Interim (all majors):** workspace bulk delete uses a **UNION** of indexed predicates in `document_read_model.rs` plus migration 128 listing indexes — see [`serving-fence-decision.md`](serving-fence-decision.md).

**Future:** optional migration inside `DO $$ … IF server_version_num >= 180000` when a measured win justifies it; until then, capability probe only.

### RETURNING OLD/NEW (outbox / change capture)

**Deferred.** PG18 `RETURNING OLD/NEW` simplifies outbox-style triggers but requires PG18-only trigger bodies. EdgeQuake’s compensation/outbox paths stay on portable `RETURNING` + explicit reads until an outbox table is productized.

### Async I/O (`io_method`)

**Ops note (not app-default):** PG18 async I/O can reduce heap-fetch latency for large sequential scans and some ANN post-filter paths. EdgeQuake does **not** set `io_method` in application code.

Suggested operator tuning (measure before/after on your fleet):

```sql
-- Session or role default — benchmark ANN + list paths first
ALTER SYSTEM SET io_method = 'io_uring';  -- or 'worker' where io_uring unavailable
SELECT pg_reload_conf();
```

Record p95/recall in your runbook; revert if neutral or regressive (LAW-I2).

**RM4 measure checklist:** before enabling fleet-wide, capture p95 for (1) filtered typed ANN with fence JOIN, (2) document list `(workspace_id, created_at)`, (3) AGE neighbor expansion — compare `io_method=worker` vs `io_uring` vs default on the same hardware. Artifacts go under `specs/091-simplify-data-layer/measurements/`.

### Skip scan + composite btrees

PG18 may choose skip scans on composite indexes where PG16/17 seq-scan. Review new composite indexes (e.g. migration 128) with `EXPLAIN (ANALYZE, BUFFERS)` on PG18 before assuming seq-scan plans — no separate SQL branch required today.

## SSOT

- Runtime probes: `edgequake-storage/src/adapters/postgres/capabilities.rs`
- Version pins: `edgequake/docker/extension-pins.sh`
- Operator matrix: [`version-matrix.md`](version-matrix.md)
