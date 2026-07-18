# SPEC-058 — Data-layer hardening

**Status:** Implemented (v0.19.x)  
**Product docs:** [Data Layer deep dive](../../docs/deep-dives/data-layer.md)

## Goal

Fix P0 integrity / isolation / FTS / cancel gaps in the PostgreSQL + AGE + pgvector storage layer using first principles: same database ≠ atomic multi-store write; retract must unindex; generated FTS must see the SSOT.

## Waves

| Wave | Topic | Primary code |
| ---- | ----- | ------------ |
| 1 | Safe compensate (created-only vectors) + fail-fast merger | `merger/mod.rs`, `compensation.rs` |
| 2 | SQL `eq_merge_graph_properties` | M090, `nodes_ops/mutate.rs`, `edges_ops.rs` |
| 3 | Writable `content_tsv` + KV populate | M091, `vector/ddl.rs`, `storage_impl.rs`, `fts.rs` |
| 4 | Cancel → `retract_document_indexes` | `retract_document_indexes.rs`, `text_insert/cancel.rs` |
| 5 | Scoped `get_incident_edges_batch` | `edges_ops.rs`, `graph_hops.rs`, Local/Global |
| 6 | Dimension mismatch fail-closed | `vector/migration.rs` |
| 7 | Local/Global `vector_type` SQL + Mix arm semaphore + `ef_construction` 64 | query modes, `config.rs` |
| 8 | Docs + metrics + this pack | `docs/deep-dives/data-layer.md` |

## Related

- [001-first-principles.md](001-first-principles.md)
- [002-test-matrix.md](002-test-matrix.md)
- Assessment canvas: storage-layer-assessment (Cursor canvases)
