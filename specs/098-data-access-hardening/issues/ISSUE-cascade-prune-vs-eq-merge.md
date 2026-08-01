# ISSUE — Cascade prune vs `eq_merge` union

| Field | Value |
|-------|-------|
| ID | F-098-20 |
| Law | LAW-098-12 |
| Severity | P0 |

## Symptom

Bulk delete of dense shared-KG documents (e.g. hyper-connection variants) fails with:

`Post-proof failed: N nodes and M edges still reference document sources`

UI honesty (Symptom D) correctly shows **Delete failed** + the reason. The cascade itself does not stick.

## Root cause

1. Shared entities/edges are pruned in memory (`remaining_sources` → `apply_rebuild_to_properties`).
2. Cascade persisted them via `upsert_nodes_batch` / `upsert_edges_batch`.
3. Postgres native `ON CONFLICT` applies `eq_merge_graph_properties`, which **unions** `source_ids` / `source_chunk_ids`.
4. Deleted document chunks reappear → `post_proof_source_absent` fails → fail-closed `delete_failed`.

Memory adapters replace properties, so memory e2e green hid the AGE bug.

## Fix

- `GraphPropertyWriteMode::{MergeSources, Replace}`
- Cascade shared updates call `upsert_*_batch_with_mode(..., Replace)` → `properties = EXCLUDED.properties`
- Ingest keeps `MergeSources` / `eq_merge_graph_properties`

## Proof

`e2e_spec098_cascade_shared_prune_survives_eq_merge` (postgres + AGE).

## Operator

After deploy, **retry Delete** on stuck `delete_failed` documents. No special repair script.
