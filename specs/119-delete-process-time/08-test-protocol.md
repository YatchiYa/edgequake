# 08 — Test Protocol

## Contract (storage)

| Test | Assert |
|------|--------|
| `contract_spec119_edge_singular_citation_indexes_exist` | Both singular btrees after ensure_indexes |
| `contract_spec119_singular_probe_uses_index` | EXPLAIN Index Cond for chunk_id, document_id, and OR (BitmapOr); cast trap documents Seq Scan |
| `contract_spec119_singular_sql_source_has_no_jsonb_cast_on_arrow` | Singular SQL fragment has no `::jsonb->>'…'` |

## E2E (storage)

| Test | Assert |
|------|--------|
| `e2e_spec119_singular_edge_discovery_wall` | 200 singular-only edges discovered under 2s; OR EXPLAIN ANALYZE uses BitmapOr + Index Cond |
| `e2e_spec119_live_graph_singular_index_cond_if_present` | When configured graph ≥10k edges + index present → Index Cond (else skip) |
| Existing `e2e_spec071_edge_source_prefix_gin` | Modern GIN unchanged |

## API / service

| Test | Assert |
|------|--------|
| `graph_cleanup_timeout` unit tests | Detection + no raw Postgres in user copy |
| `retract_checked_removes_singular_only_citation_edges` | Reprocess retract SSOT clears Symptom F edges |
| Existing memory scan_ops singular match | Still green |

## Commands

```bash
cargo test -p edgequake-api --lib graph_cleanup_timeout
cargo test -p edgequake-api --lib retract_document_indexes
cargo test -p edgequake-storage --features postgres --test contract_spec119_edge_singular_citation_indexes -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec119_singular_edge_discovery -- --nocapture
```

## Exit criteria

Contracts green; wall e2e green; retract singular green; UX SSOT unit green; modern GIN regression green.
