# SPEC-074 — Retract completeness checklist

When a document is **deleted**, **cancelled**, **orphaned**, or **failed**, every search surface for that `document_id` must clear. Dual-SSOT (relational + KV + vectors + AGE) means CASCADE on relational rows alone is **not** enough.

## Surfaces

| Surface | Expected after retract | Code / ops |
|---------|------------------------|------------|
| Relational | `documents` / `chunks` / `pdf_documents` / lineage gone or cascade | Migrations + delete API |
| KV | Chunk keys for document removed (metadata KV may remain for status sync) | Pipeline / status sync; retract does **not** wipe doc metadata KV ([`retract_document_indexes.rs`](../../edgequake/crates/edgequake-api/src/services/retract_document_indexes.rs)) |
| Vectors | `delete_by_document` removes rows by `document_id` column / JSONB / `{doc}-chunk-%` | [`storage_impl.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/storage_impl.rs) |
| AGE | Sole-source nodes/edges removed; shared sources pruned via `source_ids` / `source_chunk_ids` | SPEC-058/059 cascade |
| ANN orphans | No leftover embeddings for deleted doc; hot partial HNSW still consistent | Cancel / orphan janitor (`EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER`) |

## Compensate vs full-doc retract

| Path | Deletes |
|------|---------|
| **Retract** (`retract_document_indexes`) | All vectors for `document_id` + graph cascade |
| **Compensate** (saga merge failure) | Only **created** artifact IDs (`upsert_report_created`) — never shared neighbor updates |

## Wave-2 denorm guard

Required so partial HNSW implication stays valid:

1. Upsert metadata includes `workspace_id` and `document_id` (or `source_document_id`).
2. Materialized columns populated on INSERT (`COALESCE` from metadata).
3. Filtered search uses **columns-only** when Wave-2 on (no JSONB `OR` that breaks implication).
4. EXPLAIN on hot path: Index Scan on partial/HNSW (or DiskANN), not Seq+Sort on the workspace slice.

## Verification

```bash
cargo test -p edgequake-storage --test e2e_spec074_retract_and_denorm
# With Postgres:
# DATABASE_URL=… cargo test -p edgequake-storage --test e2e_spec074_retract_and_denorm -- --nocapture
```

API cancel retract contracts remain in SPEC-059 (`e2e_spec059_cancel_retract`).
