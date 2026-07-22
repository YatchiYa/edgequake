# 069 — Reliable Document Delete (DDL Off Hot Path + Honest UX)

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** Schema ≠ request path; delete/ingest only mutate data

## Problem

Bulk delete stalled on **Removing graph entities & edges**. Logs showed SPEC-062 eq_* DDL during Deletion tasks (`ALTER TABLE … eq_source_id`, `DROP/CREATE TRIGGER`) under the ~15s graph **query** `statement_timeout`, with multiple workers racing locks. The UI also showed hex id slices (`019f878a`) instead of filenames and had no poll fallback if WS went quiet.

## Law

| Concern | SSOT |
|---------|------|
| eq_* columns / indexes / triggers | Boot reconcile (`support/092/apply.sql` + `ensure_indexes` at init) |
| Hot-path ensure | Catalog early-exit + `indexes_verified`; single-flight Mutex |
| DDL session GUCs | `statement_timeout=0`, `lock_timeout=5s` (query path keeps `EDGEQUAKE_GRAPH_QUERY_TIMEOUT_SECS`) |
| Missing schema after boot | Fail closed: `graph schema not bootstrapped (eq_id)` |
| Delete panel name | Prefer `file_name` / `title`; never downgrade to hex |
| Progress liveness | WS phase ticks + 3s heartbeat + ~2s poll fallback |

## Changes

1. **Storage** — `indexes_verified` after boot-ready catalog; `ensure_indexes_lock`; `setup_age_ddl_session`; `eq_id_schema_ready` early-exit (no unconditional `DROP TRIGGER`); native upsert fail-closed.
2. **Migrations** — every-boot `reconcile_migration_092` (`migrations/support/092/apply.sql`). Marker `092_*.sql` stays checksum-locked.
3. **Cascade / deletion** — `cascade_remove_document_sources_with_progress` + periodic RemovingGraph heartbeats.
4. **WebUI** — `preferDocumentName` / no hex overwrite in `onMutate`; poll document/task while session active; “Still working…” on long graph phase.

## Verify

```bash
# From edgequake/
cargo test -p edgequake-storage --test contract_spec069_ddl_off_hotpath
cargo test -p edgequake-api --test contract_spec069_delete_progress
cargo test -p edgequake-api --lib state::migration_bootstrap::tests::m092_apply_sql_is_boot_owned_eq_id_ssot
cargo test -p edgequake-storage --test contract_source_prefix_discovery_gin
cargo fmt -p edgequake-storage -p edgequake-api
cargo clippy -p edgequake-storage -p edgequake-api --lib --tests -- -D warnings 2>&1 | rg "graph_lifecycle|lifecycle_ops|session|document_deletion|document_graph_cascade|m092|contract_spec069" || true

# From edgequake_webui/
pnpm exec vitest run src/lib/documents/__tests__/deletion-session.test.ts
# Prefer localhost (not 127.0.0.1) — Next.js 16 blocks cross-origin /_next assets.
PLAYWRIGHT_BASE_URL=http://localhost:3000 PLAYWRIGHT_SKIP_STACK_CHECK=1 \
  pnpm exec playwright test e2e/spec050-delete-feedback-zone.spec.ts --project=chromium
```

## Success criteria

- Bulk delete of N docs: **zero** SPEC-062 DDL warnings during Deletion tasks (DDL only at boot/reconcile).
- Panel shows real filenames; graph phase advances or shows liveness; completes or fails visibly.
- Post-proof / GIN source-prefix discovery remains green.
