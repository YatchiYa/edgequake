# 07 — Issue #5: Statement timeout on graph node counts (57014)

**Crit:** Medium · **Volume:** 4 · **Law:** LAW-I4 · **E2E:** E2E-104-05

## Symptom (prod)

Query id `DATA-AGE-GRAPH-NODE-COUNTS-BY-SOURCE-PREFIXES` canceled (`57014`). Shape: `CROSS JOIN generate_series` + JSONB `@>` on `source_ids` against `eq_eq_default_graph."Node"`.

## Why V22 has it

```ascii
 List / reconcile needs entity_count per doc
        │
        ▼
 node_counts_by_source_prefixes
   probes = prefixes × series(0..N-1)
   JOIN "Node" ON properties->source_ids @> chunk_id
        │
        ├─ WITH M038 idx_node_source_ids_gin ──▶ Bitmap Index Scan (~ms)
        │
        └─ WITHOUT GIN / cold / huge batch
                 │
                 ▼
           Seq / Join Filter ──▶ > statement_timeout (300ms SPEC-089)
                 │
                 ▼
           57014; soft-fail partial map (list still returns)
```

SPEC-089 (in ≤0.22 line) already: child `"Node"`, MATERIALIZED probes, batch ≤32, 300ms `SET LOCAL`, page-scoped reconcile. Timeouts remain possible when GIN missing or graph huge.

## V23 residual

**Mitigated, not eliminated.** SPEC-091 did not replace this op.

## Remediation

1. **Ops:** verify per graph:

```sql
SELECT indexname FROM pg_indexes
WHERE schemaname = 'eq_eq_default_graph'
  AND indexname = 'idx_node_source_ids_gin';
```

2. **Inspector:** schema finding when M038 GIN absent on configured graph (LAW-I2 + I4).
3. Keep SPEC-089 bounds; do not raise timeout without measurement proof.
4. Rely on existing e2e: `e2e_issue331_*`, `e2e_issue336_*`, plus E2E-104-05 smoke.

## Fix status (2026-08-03)

**Observable, not cured.** Grade B+. Migration impact: **none** (M038 already in 0.22 line). Missing GIN → Warning on inspect; 57014 under load still possible (EC-13 OPS).

## Ops note

If prod still 57014 after GIN present: capture EXPLAIN ANALYZE on a single batch of 32 prefixes; escalate as capacity, not as “wrong table name” (#2 is separate — that query used the **correct** graph in the timeout samples).

## SPEC-089 handoff (measurement-gated; no timeout raise here)

SPEC-104 closed **naming** and **GIN observability** only:

- Inspector discovers all `eq_%_graph` and warns when `idx_node_source_ids_gin` is missing.
- Staging spot-check (2026-08-03): GIN **present** on `eq_eq_default_graph` — see [`measurements/v23-sql-spotchecks.txt`](measurements/v23-sql-spotchecks.txt).

Remaining `57014` with GIN present is a **capacity** residual. Owners: SPEC-089 / [GH-336](../089-health-check/issues/GH-336-health-pool-cross-join.md).

Before any product change (batch size, `statement_timeout`, denormalized `document_id`):

1. EXPLAIN ANALYZE one reconcile batch (≤32 prefixes) on a production-sized graph.
2. Confirm pool pressure is not the primary cause (LAW-H1 / GH-336 misattribution).
3. Raise timeout or widen batch **only** with measurement proof — do not land speculative timeout bumps in SPEC-104.
