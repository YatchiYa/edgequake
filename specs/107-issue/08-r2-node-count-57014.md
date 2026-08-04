# 08 — R2 Node-count `57014` under load (first principles)

> **Residual:** SPEC-107 R2 / SPEC-104 #5 / EC-13  
> **Laws:** LAW-H1..H5 ([SPEC-089](../089-health-check/00-first-principles.md)), LAW-I4 ([SPEC-104](../104-fix-datalayer/01-first-principles.md))  
> **Product:** pin **v0.24.1** — no `statement_timeout` raise in this harden

## Verdict

SQLSTATE **`57014`** (`query_canceled`) on `DATA-AGE-GRAPH-NODE-COUNTS-BY-SOURCE-PREFIXES` is a **work-budget** residual, not a wrong-graph-name bug. List path was already SPEC-089-bounded. INV-C was the remaining LAW-H1 cliff (≤50 prefixes in one 300ms statement).

**This harden:** INV-C chunks with public `SOURCE_PREFIX_BATCH_LIMIT` (32) + shared timeout SSOT; list soft-fail warn tags the dataop; measurement gate before any timeout/batch/denorm change.

```ascii
 Complexity (per statement)
   probes = |prefixes| × probe_limit
   list:   ≤32 × ≤256 = 8192  + GIN @> + SET LOCAL 300ms
   INV-C before: ≤50 × ≤256 in ONE txn  ← cliff
   INV-C after:  chunks(32) × ≤256      ← LAW-H1 aligned
 Ideal with GIN: ~O(P log N); without / wrong plan: SeqScan → 57014
```

## Axioms

1. **Postgres cancels statements that exceed `statement_timeout`** — `57014` means the kill worked (LAW-H2), not that the product is “broken.”
2. **Cartesian probes are only safe when batched** — `|prefixes| × probe_limit` must stay under a fixed budget per statement (LAW-H1).
3. **GIN `@>` on child `"Node"` is the indexed path** — never replace with LIKE on the hot path (LAW-H4 / GH-331).
4. **Write-path `entity_count` is SSOT** — list reconcile is a soft safety net (LAW-H5).

## Root-cause ranking

| Rank | Cause | Status |
|------|-------|--------|
| 1 | INV-C one-shot ≤50 prefixes vs analytics `chunks(32)` | **Closed** (this harden) |
| 2 | Intrinsic CTE cost + pool contention | Soft-fail / Warning |
| 3 | Missing M038 GIN | Inspect Warning (SPEC-104 #5) |
| 4 | True scale with GIN present | OPS residual — Phase-2 denorm |

## Decision gate (before product timeout/batch change)

1. `EXPLAIN (ANALYZE, BUFFERS)` one reconcile batch (≤32 prefixes) on a production-sized graph.
2. Confirm `Bitmap Index Scan` on `idx_node_source_ids_gin` (not Join Filter / SeqScan).
3. Confirm pool pressure is not the primary cause (GH-336 misattribution).
4. Only then consider timeout raise, wider batch, or denormalized `document_id` ([SPEC-089 Phase 2](../089-health-check/00-first-principles.md)).

## Ops checklist

```sql
SELECT indexname FROM pg_indexes
WHERE schemaname = 'eq_eq_default_graph'
  AND indexname = 'idx_node_source_ids_gin';
```

Grep logs: `DATA-AGE-GRAPH-NODE-COUNTS-BY-SOURCE-PREFIXES`, `57014`, `inv_c_gin_batch`, `P-A3:`.

## Partner note

List documents still return 200 with KV/`entity_count` on soft-fail. Hourly INV-C emits Warning when a batch fails with empty map. Prefer upgrade + GIN verify before capacity work.

## Cross-refs

| Doc | Role |
|-----|------|
| [SPEC-104 #5](../104-fix-datalayer/07-issue-05-node-counts-timeout.md) | RCA / “observable not cured” |
| [GH-336](../089-health-check/issues/GH-336-health-pool-cross-join.md) | Pool / CROSS JOIN history |
| [07-residual-risks.md](07-residual-risks.md) | R2 status board |
| `e2e_issue331_*` / `e2e_issue336_*` | GIN locality + boundedness |
