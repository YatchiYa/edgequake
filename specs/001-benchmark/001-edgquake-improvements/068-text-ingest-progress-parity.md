# 068 — Text/MD Ingest Progress Parity

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** One progress identity per job = server `task_id` (SPEC-054), including text/MD admits

## Problem

Uploading Markdown showed green/Done-ish UI with **“Processing pending…”** while the backend processed successfully. Terminal showed repeated:

`GET /api/v1/ingestion/insert-…/progress` → **404 Track not found**

Root cause: FE polls `task_id` (`insert-*`) per SPEC-054, but text admit stored metadata `track_id` as `upload_*` / client batch id. Staging metadata was invisible when the wsdoc index was non-empty. WS `ChunkProgress` was dropped because `startTracking` ran only after a successful poll.

## Law

| Concern | SSOT |
|---------|------|
| Progress / cancel / WS key | `task_id` = `insert-*` = metadata `track_id` |
| Client batch correlation | `client_track_id` only |
| In-flight visibility | Progress load includes `staging:{doc}-metadata` |
| FE hydrate | `startTracking` before first poll |

## Changes

1. [`document_admission.rs`](../../../../edgequake/crates/edgequake-api/src/handlers/documents/upload/document_admission.rs) — create Insert task before metadata write; `track_id`/`task_id` = `insert-*`; keep `client_track_id`.
2. [`document_metadata_scan.rs`](../../../../edgequake/crates/edgequake-api/src/services/document_metadata_scan.rs) — `load_scoped_document_metadata_for_progress`.
3. [`ingestion.rs`](../../../../edgequake/crates/edgequake-api/src/handlers/ingestion.rs) — match `track_id` or `task_id`; use staging-aware load.
4. WebUI — early `startTracking`, queued messaging (no “Processing pending…”), soft 404 admit race.

## Verify

```bash
# From edgequake/
cargo test -p edgequake-api --test contract_068_text_ingest_progress
cargo test -p edgequake-api --lib handlers::ingestion
cargo fmt --check -p edgequake-api
# Clippy on touched paths should be clean (crate may still have pre-existing -D warnings elsewhere)
cargo clippy -p edgequake-api --lib --tests -- -D warnings -A dead_code 2>&1 | rg "document_admission|document_metadata_scan|handlers/ingestion|contract_068" || true

# From edgequake_webui/
pnpm exec vitest run \
  src/lib/upload/__tests__/perform-file-upload.test.ts \
  src/lib/upload/__tests__/progress-track-id.test.ts \
  src/hooks/__tests__/use-ingestion-progress-068.test.ts
# Prefer localhost (not 127.0.0.1) — Next.js 16 blocks cross-origin /_next assets.
PLAYWRIGHT_BASE_URL=http://localhost:3025 PLAYWRIGHT_SKIP_STACK_CHECK=1 \
  pnpm exec playwright test e2e/spec068-text-ingest-progress.spec.ts --project=chromium
```
