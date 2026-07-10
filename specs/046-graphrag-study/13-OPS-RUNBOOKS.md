# Ops Runbooks — Postgres / pgvector / AGE (SPEC-046 OPS-P2.18)

**Audience:** on-call / platform  
**Pins SSOT:** `edgequake/docker/extension-pins.sh`  
**Related:** [10-POSTGRES-PGVECTOR-AGE-PERFORMANCE.md](./10-POSTGRES-PGVECTOR-AGE-PERFORMANCE.md), [09-OPS-RELIABILITY-DEEPSTUDY.md](./09-OPS-RELIABILITY-DEEPSTUDY.md)

---

## 1. Major version upgrade (PG16 → 17 → 18)

1. Build target image: `EQ_POSTGRES_PROFILE=pg17 make postgres-image-build` (or pg18).
2. Snapshot / dump: `pg_dump` logical dump of `edgequake` DB.
3. Stop writers: `make stop` (or drain API).
4. Start new major container; restore dump; run `make backend-bg` (sqlx + reconcile).
5. Verify `/ready` has empty `blockers`; check `missing_hnsw_index` absent.
6. Smoke: `bash specs/046-graphrag-study/e2e/run_ops17_perf_smoke.sh`.
7. AGE note: PG16 uses AGE 1.6; PG17/18 use AGE 1.7 — expect longer `CREATE EXTENSION` on first boot.

**Rollback:** restore previous image + dump; do not mix AGE majors on same data directory without dump/restore.

---

## 2. Suspected index / catalog corruption

**Symptoms:** query timeouts, `ERROR: index ... contains unexpected zero page`, ANN returning empty under filter.

1. Confirm pgvector version: `SELECT extversion FROM pg_extension WHERE extname = 'vector';` (need ≥ 0.8.0 for iterative_scan).
2. Check HNSW presence: admin `/admin/storage/inspect` or `missing_ann_index_tables` on `/ready`.
3. **REINDEX** (maintenance window):
   ```sql
   REINDEX INDEX CONCURRENTLY <hnsw_index_name>;
   ```
4. If AGE graph indexes missing: run support script for M038 / `ensure_indexes` path (see migration bootstrap logs).
5. After repair: restart backend so reconcile re-checks ANN; confirm `/ready` 200.

---

## 3. Storage drift SLO (OPS-19)

**Metric:** `edgequake_storage_drift_critical` (gauge), `edgequake_storage_drift_violations_total`.

| Condition | Action |
|-----------|--------|
| `edgequake_storage_drift_critical > 0` for 15m | Page: run `/admin/storage/inspect`, then SAFE `/admin/storage/repair` |
| Warning-only drift | Ticket within 1 business day; prefer SAFE auto-repair from hourly monitor |
| INV-C entity_count drift > 20% sample | Investigate merge/compensation failures; check quarantine logs |

Hourly monitor already logs CRITICAL and applies SAFE repairs (`StorageInspector::spawn_hourly_monitor`).

---

## 4. Chunk retry after partial extract

1. `GET /api/v1/documents/{id}/failed-chunks`
2. `POST /api/v1/documents/{id}/retry-chunks` — merges via `KnowledgeGraphMerger` (OPS-21).
3. If KV chunk missing → status `abandoned`; re-ingest document.

---

## 5. Quick health checklist

```bash
curl -s http://localhost:8080/health | jq .
curl -s http://localhost:8080/ready | jq .
curl -s http://localhost:8080/metrics | grep -E 'drift|ann_index|popular_node|sparse_retrieval|faithfulness'
```
