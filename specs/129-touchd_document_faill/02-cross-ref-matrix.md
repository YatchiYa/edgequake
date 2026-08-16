# 02 — Cross-ref matrix

| Ref | Role for SPEC-129 |
|-----|-------------------|
| [#381](https://github.com/raphaelmansuy/edgequake/issues/381) | Symptom: touch CHECK violation on resume |
| [#377](https://github.com/raphaelmansuy/edgequake/issues/377) | Often leaves crash checkpoint → resume path; **out of fix scope** |
| [#374](https://github.com/raphaelmansuy/edgequake/issues/374) | Prior legacy_vector_id race context for #377 |
| SPEC-047 P1 | Early `touch_document_status` dual-write |
| SPEC-047 P5 | Slim checkpoint; embeddings omitted; resume re-embed |
| SPEC-057 P2 | Honest `re_embedding` stage string |
| SPEC-098 / LAW-098-9/11 | Migration 141 CHECK + lifecycle statuses |
| SPEC-083 C-23 | `completed` ↔ `indexed` terminal equivalence |
| SPEC-021 P-A1 | `update_document_stats` relational refresh |
| `normalize_documents_column_status` | Existing shell A→B mapper |
| `relational_documents_status_for_write` | New write-path SSOT (this spec) |

```ascii
  SPEC-057 honesty ──► KV status = re_embedding
         │
         ▼
  SPEC-047 P1 touch ──► must LAW-129-2 project ──► CHECK (141)
         │
         X (bug) raw copy
```

## Doc ↔ code anchors

| Concern | Path |
|---------|------|
| Resume sets `re_embedding` | `edgequake-api/.../text_insert/extraction.rs` |
| P1 touch call | `edgequake-api/.../status_updates.rs` |
| Postgres touch | `edgequake-storage/.../pdf_storage_impl.rs` |
| Memory touch | `edgequake-storage/.../memory/pdf.rs` |
| Sidecar touch | `edgequake-api/.../task_document_sync.rs` |
| Shell normalize | `edgequake-storage/.../document_shell.rs` |
| CHECK DDL | `migrations/141_spec098_document_lifecycle_status.sql` |

## Cross-refs

- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
