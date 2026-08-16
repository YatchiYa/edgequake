# Lens 002 — Full Stack Developer

## Problem shape

Three near-identical incomplete maps (`completed`→`indexed` only) in:

- `pdf_storage_impl::touch_document_status`
- `memory/pdf::touch_document_status`
- `task_document_sync::touch_relational_document_track_status_best_effort`
- plus `status_updates::refresh_relational_document_stats`

Shell already has the full vocabulary map.

## Implementation rules

1. Add `relational_documents_status_for_write` once; re-export.
2. Replace every incomplete map with a call to that helper.
3. Keep extraction KV write as `re_embedding`.
4. Contract-test source contains helper name on touch paths.
5. e2e: raw UPDATE fails; trait touch succeeds.

## Failure modes to watch

- Import cycles (API → storage OK; avoid storage → API).
- Forgetting memory adapter (tests pass on memory, fail on PG).
- Stats path writing raw stage during finalize.

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
