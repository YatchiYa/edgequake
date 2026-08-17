# ISSUE — EDGE ON CONFLICT cardinality (SQLSTATE 21000)

## Repro (observed)

- UI: Documents → reprocess large multi-chunk extract (e.g. `xHC Expanded Hyper-Connections.extracted.md`) → **Failed**
- Error: `Knowledge graph persist failed: Graph error: 1 knowledge-graph merge error(s) during persist: Storage error: Database error: Native SQL edge batch upsert failed: … ON CONFLICT DO UPDATE command cannot affect row a second time`
- Graph merge / entity extraction may succeed for earlier chunks; persist fails on a batch with duplicate arbiter keys or dual UNIQUE schema drift.

## Root cause

PostgreSQL 16/17/18 forbid one `INSERT … ON CONFLICT DO UPDATE` from affecting the same target row twice (deterministic upsert / SQLSTATE `21000`). Proposed EDGE rows collide on the arbiter `(eq_source_id, eq_target_id, eq_rel_type)` after BEFORE INSERT sync triggers, and/or a legacy endpoint-only UNIQUE (`idx_edge_source_target_unique` / 2-col `idx_edge_eq_source_target`) collapses multigraph batches.

Contributing debt (F-098-06…09):

1. Within-batch duplicate `(src, tgt, rel)` without LWW dedupe / SQL `DISTINCT ON`.
2. Boot early-exit that left legacy UNIQUEs in place.
3. Cypher MERGE keyed only on `(source_id, target_id)` (multigraph split-brain when native writes off).
4. Native DO UPDATE dropping `eq_merge_graph_properties` (SPEC-058 regression — separate but related harden).

## Fix (SPEC-098 W6–W8)

1. Single EDGE arbiter only; drop legacy UNIQUEs every boot + migration 140.  
2. Rust LWW dedupe + SQL `DISTINCT ON` with trigger-aligned casefold.  
3. Restore `eq_merge_graph_properties` on native node/edge DO UPDATE.  
4. Cypher MERGE (batch + single-edge) includes normalized `relation_type`.  
5. E2E: native ON cardinality, legacy UNIQUE reconcile, native-off Cypher multigraph, perf.

## Acceptance

- Reprocess the failing document → **Completed**; no SQLSTATE 21000.  
- `pg_indexes` on AGE `EDGE`: `idx_edge_eq_source_target_rel` present; legacy 2-col / expression UNIQUEs absent.  
- Native **and** Cypher (`EDGEQUAKE_NATIVE_GRAPH_WRITES=0`) accept duplicate + multigraph batches.
