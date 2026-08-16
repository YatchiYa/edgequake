# 03 — Code as-is (pre-fix)

## Call graph (#381)

```ascii
  P7e load checkpoint (needs_reembed=true)
    → update_document_status("re_embedding")     extraction.rs
        → KV upsert status=re_embedding            status_updates.rs  OK
        → touch_relational_document_status(...)    status_updates.rs
            → touch_document_status(..., "re_embedding")
                → UPDATE documents.status = 're_embedding'   ← CHECK FAIL
                → WARN "SPEC-047 P1: touch_document_status failed"
```

## Failing writers (partial map only)

**Postgres touch** (`pdf_storage_impl.rs`):

```rust
let pg_status = if status == "completed" {
    "indexed"
} else {
    status  // ← re_embedding passes through
};
```

**Memory touch** (`memory/pdf.rs`): identical incomplete map.

**Sidecar** (`task_document_sync.rs`): identical incomplete map.

**Stats refresh** (`status_updates.rs` `refresh_relational_document_stats`): identical incomplete map before `update_document_stats`.

## Working mapper (shell only)

`document_shell.rs` `normalize_documents_column_status`:

```ascii
  re_embedding | merging | storing | …  →  processing
  queued                                  →  pending
  partial_success                         →  partial_failure
  deleting / delete_failed                →  passthrough
  CHECK allowlist values                  →  passthrough
```

Touch never called this function → DRY violation → #381.

## Honest stage (keep)

`extraction.rs` when `result.needs_reembed()`:

```rust
self.update_document_status(
    &document_id,
    "re_embedding",
    Some("Re-generating embeddings after slim checkpoint (embeddings_omitted)"),
).await?;
```

Contract `pipeline_checkpoint.rs::re_embedding_stage_string_is_honest` requires this string remain in extraction.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Matrix: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
