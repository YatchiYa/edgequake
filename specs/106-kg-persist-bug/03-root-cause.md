# 03 — Root Cause (#356)

## Five whys

1. Persist fails with merge error → storage batch query fails.
2. Batch query is `get_edges_for_nodes_batch` during relationship merge.
3. SQL JOINs `vids.vid` to `EDGE.start_id` / `end_id` as raw `graphid`.
4. AGE does not provide a usable `graphid = graphid` operator (same as #214).
5. #214 remediated degrees only; this call site was never updated → still on v0.24.0.

## Non-causes

- Native upsert `ON CONFLICT` / Cypher `MERGE` (failure is **before** upsert).
- Wrong graph name (SPEC-104 #2) — different errcode (`42P01`).
- Missing `search_path` alone — types are schema-qualified; operator still missing.

## Code (pre-fix)

`edgequake/crates/edgequake-storage/src/adapters/postgres/graph/nodes_ops/read.rs` — `JOIN vids src ON src.vid = e.start_id`.

## Code (post-fix)

`JOIN vids src ON src.vid_text = e.start_id::text` (and end_id).
