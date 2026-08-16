# 04 — Target architecture

## Principle

```ascii
                    ┌─────────────────────────────────┐
                    │  update_document_status(raw)    │
                    └────────────┬────────────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
         KV metadata      WS / UI chips      touch / stats / sidecar
         status=raw         stage=raw                │
                                                     ▼
                                   relational_documents_status_for_write
                                                     │
                                   normalize_documents_column_status
                                                     │
                                   completed → indexed (list preference)
                                                     │
                                                     ▼
                                          UPDATE documents.status
                                          (CHECK-safe always)
```

## New SSOT

```rust
pub fn relational_documents_status_for_write(raw: &str) -> String {
    let normalized = normalize_documents_column_status(raw);
    if normalized == "completed" {
        "indexed".to_string()
    } else {
        normalized
    }
}
```

Lives in `document_shell.rs` next to `normalize_documents_column_status`. Re-exported from `edgequake_storage` for API crate use.

## Writers that must call it

| Writer | File |
|--------|------|
| `PdfStorage::touch_document_status` | postgres + memory |
| `update_document_stats` (defense) | postgres (+ memory if applicable) |
| Sidecar `touch_relational_document_track_status_best_effort` | task_document_sync |
| `refresh_relational_document_stats` status arg | status_updates |

## What does not change

- KV still stores rich stages (`re_embedding`, …)
- CHECK DDL unchanged (no migration)
- Touch remains best-effort (errors still non-fatal if DB down / missing row)

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| SRP | Storage owns CHECK projection; pipeline owns stage honesty |
| OCP | New UI stages → extend normalizer match arms only |
| DRY | One function; delete duplicated `if completed { indexed }` blocks |
| DIP | API depends on exported helper, not copy-pasted maps |

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Laws: [01-first-principles.md](01-first-principles.md)
