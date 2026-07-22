# 071 — Lineage / Edge Source-Prefix Discovery (Scalability)

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** Index locality; modern-first; O(probes); measure; fail soft on UX path

## Incident

```text
GET /api/v1/lineage/documents/{id} → 500 STORAGE_ERROR (~25s)
Source-prefix edge query failed: canceling statement due to statement timeout
```

Adjacency (`get_edges_for_nodes_batch` on `"EDGE"`) was already indexed. Lineage then always merged `find_edges_by_source_prefixes`, which:

1. Scanned AGE parents `_ag_label_edge` / `_ag_label_vertex` while GIN lived on child `"EDGE"` / `"Node"`.
2. Always ran legacy LIKE / `source_chunk_ids` SeqScan after modern — alone enough to burn `statement_timeout`.

SPEC-070 closed **Node** EXPLAIN; the edge path slipped through.

## Laws

| Law | Meaning |
|-----|---------|
| Index locality | Query the table that owns the index (`"Node"` / `"EDGE"`) |
| Modern-first | GIN `@>` JOIN on `source_ids` only; legacy residual / opt-in |
| O(probes) | Cost ∝ probe set, not full edge table |
| Measure | EDGE EXPLAIN + lineage wall budget |
| Fail soft (UX) | Lineage returns adjacency on prefix timeout; delete stays fail-closed |

## Changes

1. **Storage** (`scan_ops.rs`): modern discovery `FROM {graph}."Node"` / `"EDGE"`; edge endpoints via `eq_source_id` / `eq_target_id`.
2. **Legacy**: only when `EDGEQUAKE_SOURCE_PREFIX_LEGACY=1` (default off). No `ORDER BY` on legacy.
3. **API** (`find_relationships_for_document_lineage`): best-effort prefix merge; warn + adjacency SSOT on error.
4. **Gates**: contracts + `e2e_spec071_edge_source_prefix_gin` + lineage unit inject-fail.
5. Removed unused giant-OR `build_source_prefix_clause_modern`.

Discovery result sets remain capped at `LIMIT 5000`.

## Verify

```bash
cargo test -p edgequake-storage --test contract_source_prefix_discovery_gin
cargo test -p edgequake-storage --lib source_prefix_clause_tests
cargo test -p edgequake-api --lib lineage_relationships_survive_source_prefix_timeout

# Live (DATABASE_URL required):
cargo test -p edgequake-storage --features postgres --test e2e_spec071_edge_source_prefix_gin -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec070_delete_no_ddl -- --nocapture
```

## Success criteria

1. Lineage for large docs returns 200 without source-prefix edge timeout.
2. EXPLAIN edge discovery uses `idx_edge_source_ids_gin` (Bitmap/Index).
3. Legacy SeqScan is not on the default request path.
4. Lineage degrades to adjacency-only on prefix failure; delete cascade still fail-closed.
5. Contracts + live e2e green; SPEC-070 scorecard discovery row points here.
