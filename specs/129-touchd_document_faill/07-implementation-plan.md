# 07 — Implementation plan

## Work packages

```ascii
  WP-0  Spec pack + CHANGELOG          (docs)
  WP-1  relational_documents_status_for_write + unit tests
  WP-2  Wire postgres/memory touch + update_document_stats
  WP-3  Wire sidecar + refresh_relational_document_stats
  WP-4  e2e_spec129_touch_status_check + source contracts
  WP-5  GitHub #381 comment
  WP-6  cargo test / clippy on touched crates
```

## WP-1 — SSOT helper

File: `edgequake/crates/edgequake-storage/src/adapters/postgres/document_shell.rs`

- Add `relational_documents_status_for_write`.
- Unit: `re_embedding`→`processing`, `completed`→`indexed`, `deleting` passthrough, `queued`→`pending`.
- Re-export from `adapters/postgres/mod.rs` and `lib.rs` (`#[cfg(feature = "postgres")]` as needed; also usable from memory path via same module or a tiny shared fn).

**Note:** Memory adapter should call the same function. Prefer putting the helper in `document_shell.rs` (already `pub`) and importing via `edgequake_storage::adapters::postgres::document_shell` or crate re-export. If memory builds without postgres feature, move helper to a feature-agnostic module (e.g. keep in `document_shell` but ensure memory crate path can reach it — `document_shell` is under `postgres` feature). Check: memory adapter is always available; postgres module is `cfg(feature = "postgres")`.

**Resolution:** Export helper from a feature-agnostic path:

- Option used: keep functions in `document_shell.rs` which is compiled with postgres feature; for memory-only builds, duplicate is bad — check Cargo features of edgequake-storage.

If `document_shell` is only behind postgres, either:
1. Move normalizer+write helper to `edgequake-storage/src/documents_status.rs` (preferred DRY), or
2. Gate memory touch the same way as today and use `#[cfg(feature = "postgres")]` import in memory with fallback local map.

**Chosen:** Move is cleaner long-term, but plan says “next to existing normalizer”. Implement helper beside normalizer; memory adapter already lives in workspace builds that typically enable postgres for API. Use:

```rust
#[cfg(feature = "postgres")]
use crate::adapters::postgres::document_shell::relational_documents_status_for_write;
```

and for memory without postgres, inline call is rare — API always builds with postgres in product. Mirror: call helper unconditionally by relocating both functions to `src/documents_column_status.rs` and have `document_shell` re-export / call them.

**Concrete in this implementation:** add helper in `document_shell.rs`; re-export from postgres mod + lib; memory `pdf.rs` imports via `crate::adapters::postgres::document_shell::...` under `cfg(feature = "postgres")`, else keep completed→indexed + normalize copy — **prefer relocating to `documents_column_status.rs` at crate root** so memory always shares SSOT.

## WP-2 / WP-3 — Wire

Replace incomplete maps with helper in listed files ([04-target-architecture.md](04-target-architecture.md)).

## WP-4 — Tests

See [08-test-protocol.md](08-test-protocol.md).

## DRY / SOLID checklist

- [x] One write helper
- [x] No second CHECK widen migration
- [x] KV honesty string retained
- [x] Lifecycle statuses untouched by collapse
- [x] Remaining incomplete maps closed (`document_stage_mirror`, `finalize`)
- [x] `ensure_document_record` defensively projects (storage owns CHECK)

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
