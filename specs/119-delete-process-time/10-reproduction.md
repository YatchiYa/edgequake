# 10 — Reproduction

## Goal

Reproduce [GitHub #375](https://github.com/raphaelmansuy/edgequake/issues/375) against local EdgeQuake PostgreSQL: prove singular-edge citation lookup Seq Scans `"EDGE"` and would miss/defeat btree without expression alignment.

## Environment (2026-08-11)

| Item | Value |
|------|-------|
| Database | `postgresql://edgequake@localhost:5432/edgequake` |
| Graph | `eq_eq_default_graph` |
| Edge count | **69,405** |
| Existing singular btrees | **none** |
| Plural GIN | `idx_edge_source_chunk_ids_gin` present |
| Discovery budget | 2000ms (`SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS`) |

## Steps

### 1. Confirm missing indexes

```sql
SELECT indexname FROM pg_indexes
WHERE schemaname = 'eq_eq_default_graph'
  AND indexname ILIKE '%source_chunk_id%';
-- Has: idx_edge_source_chunk_ids_gin (plural)
-- Missing: idx_edge_source_chunk_id / idx_edge_source_document_id
```

### 2. EXPLAIN singular probe (as-is cast)

```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT ag_catalog.agtype_to_json(e.properties) AS props
FROM eq_eq_default_graph."EDGE" e
WHERE (ag_catalog.agtype_to_json(e.properties))::jsonb->>'source_chunk_id'
      = 'nonexistent-probe-id'
LIMIT 5000;
```

**Result (observed):**

```text
Seq Scan on "EDGE" e
  Filter: (((ag_catalog.agtype_to_json(properties))::jsonb ->> 'source_chunk_id') = ...)
  Rows Removed by Filter: 69405
Execution Time: ~250ms
```

At ~220k edges (reporter), linear growth crosses the 2s discovery timeout.

### 3. Prove cast defeats existing btree pattern

```sql
-- Matches idx_edge_source_id → Index Scan
EXPLAIN SELECT 1 FROM eq_eq_default_graph."EDGE" e
WHERE ag_catalog.agtype_to_json(e.properties)->>'source_id' = 'x';

-- ::jsonb → Seq Scan (same property, different expression)
EXPLAIN SELECT 1 FROM eq_eq_default_graph."EDGE" e
WHERE (ag_catalog.agtype_to_json(e.properties))::jsonb->>'source_id' = 'x';
```

### 4. Sample real singular citations

Many edges still carry singular props (example):

```text
source_chunk_id    = 019fb639-…-chunk-14
source_document_id = 019fb639-…
count ≈ 101 edges for that pair
```

So the Symptom F probe is not hypothetical on this fleet.

## Verdict

| Layer | Finding |
|-------|---------|
| Index gap | Confirmed — no singular citation btrees on `"EDGE"` |
| Plan shape | Seq Scan + JSON extract Filter |
| Cast trap | Confirmed — `::jsonb` defeats btree expression indexes |
| Parent DDL | Wrong surface — queries use child `"EDGE"` |
| Error string | Matches #375 when discovery exceeds 2s |

## After-fix verification (2026-08-11)

Applied `idx_edge_source_chunk_id` / `idx_edge_source_document_id` on `eq_eq_default_graph."EDGE"` (same DDL as `ensure_indexes`).

```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT 1 FROM eq_eq_default_graph."EDGE" e
WHERE ag_catalog.agtype_to_json(e.properties)->>'source_chunk_id'
      = '019fb639-1324-7d03-bfd7-2415b7fe7a3b-chunk-14';
```

**Result:**

```text
Index Scan using idx_edge_source_chunk_id on "EDGE" e
  Index Cond: ((ag_catalog.agtype_to_json(properties) ->> 'source_chunk_id') = ...)
  rows=101
Execution Time: ~0.3ms
```

(Previously Seq Scan ~288ms on the same probe.)
