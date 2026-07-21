# Ablation — B3b workspace-scoped AGE identity (INVALID)

**Archive:** `smoke-20260720T081853Z`  
**Profile:** `B3b_ws_scoped_graph_md_glean_v1`  
**STRUCTURE_INDUCE:** off (first principles)  
**A1 concurrency:** ≤4  

## Verdict: do not promote

EQ Acc **0.658** · valid=False (`empty_context_rate=0.125`) · LR Acc 0.779.

## Root cause (ops, not identity heuristic)

Postgres merge failed mid-relationship batch:

```
could not write to file "base/pgsql_tmp/...": No space left on device
```

Saga compensation then deleted **4228** AGE nodes + all entity/chunk vectors for WS
`5daf07b4-6824-4548-8780-54b9bc93c70c`. Acc queries raced the rollback (partial hits → empty
context). Post-run audit: **0** AGE nodes / **0** entity vectors.

Positive signal before wipe: scoped graph write path created ~4228 WS nodes (identity fix
exercised), then compensation removed them by scoped `node_id` list.

## Follow-ups

1. Free host/Docker disk (≥20 Gi) before Acc force-ingest.
2. Fail-closed wait: never treat `indexed`+storage errors as ready; gate warm pointer on
   successful ingest density.
3. Re-run `make bench001-b3b-reingest` with warm restored to B2 until promote.
