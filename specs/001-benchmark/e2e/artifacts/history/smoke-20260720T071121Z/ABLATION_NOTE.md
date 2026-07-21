# Ablation — B2 WS + A1 (INVALID — pool timeout)

**Workspace:** `e0270f5f-0b6c-4e90-882f-5f9b0eac8cff`  
**valid:** False (`empty_answer_rate=0.125`)

Five EQ answers empty because queries returned HTTP 500 `STORAGE_ERROR` / Postgres **pool timed out** under `query_concurrency=8` (not an ingest packing defect). Context field stores the error string.

**Do not use for Acc promote.** Clean retry: [`smoke-20260720T071732Z`](../smoke-20260720T071732Z/).
