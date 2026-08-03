# ISSUE — Delete / bulk-delete list dual-SSOT

## Repro (observed)

- UI: Documents → select ~5 docs (e.g. `xhc_expanded_hyper_connections.md`, `xHC Expanded Hyper-Connections*.md/pdf`) → Delete → Confirm.
- Feedback zone: **Deleting N document(s)** with spinner + subtext **Document removed**.
- Documents table: same rows still **Completed / Ready** with entity counts.
- Refresh / poll does not reconcile the two surfaces until cascade finishes (or never, if FE falsely completed sessions).

## Root cause

CQRS list read model (KV metadata + `public.documents`) diverges during delete:

1. **F-098-12** — `merge_document_summaries` does not treat KV `deleting` as inflight → SQL `completed`/`indexed` overwrites.
2. **F-098-13** — `documents_valid_status` CHECK omitted `deleting`/`delete_failed` → SQL mirror impossible.
3. **F-098-14** — Batch admit enqueues `BatchDeletion` without per-doc KV/SQL `deleting`.
4. **F-098-15** — FE: shared `batch_track_id` poll → `applyDeletionCompleted` with zero stats (“Document removed”); no delete pin; table dimming not session-driven.

## Fix (SPEC-098 W9–W11)

1. Migration 141: extend CHECK with `deleting` / `delete_failed`.  
2. Shared `admit_documents_deleting` (KV + SQL) for single + batch after durable enqueue.  
3. List merge: lifecycle inflight includes `deleting`; `delete_failed` is terminal failure.  
4. FE: pin deleting; session-driven table dimming; batch completion only on absence / per-id proof.  
5. E2E + Playwright gates wired in CI.

## Acceptance

- Mid single/batch delete: table + `GET /documents` show `deleting` (not Completed/Ready).  
- Feedback and table agree; no permanent “Document removed” while the row is still listed.  
- On success: row gone from list, KV, and SQL. On fail: `delete_failed` visible.  
- Migration 141 + checksums; CI green.
