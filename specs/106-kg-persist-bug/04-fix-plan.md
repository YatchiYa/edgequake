# 04 — Fix Plan (executed)

## Change

File: `edgequake/crates/edgequake-storage/src/adapters/postgres/graph/nodes_ops/read.rs`  
Function: `pg_get_edges_for_nodes_batch`

1. Select `n.id::text AS vid_text`.
2. Join `src.vid_text = e.start_id::text` and `tgt.vid_text = e.end_id::text`.
3. Document LAW-G1 / #356 in WHY comment (parity with degrees method).

## Acceptance

- [x] No `src.vid = e.start_id` in tree
- [x] E2E-106-01 passes on AGE Postgres
- [x] E2E-106-02 source guard passes
- [x] Similar-site audit shows zero remaining raw joins
