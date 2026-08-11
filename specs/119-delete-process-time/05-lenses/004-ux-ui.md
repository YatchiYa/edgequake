# Lens 004 — UX / UI Designer

## Current pain

Delete/reprocess surfaces opaque storage errors:

```text
Deletion failed: Storage error: Database error:
  Source-prefix singular edge query failed: …
  canceling statement due to statement timeout
```

Users cannot tell whether to retry, wait, or contact an admin.

## Target experience

```ascii
  Delete → Processing → Completed
                 │
                 └─ on discovery timeout:
                      Title: Graph cleanup timed out
                      Body:  The knowledge graph is large; cleanup did not finish in time.
                             Retry delete/reprocess. If it keeps failing, contact an admin.
                      Actions: Retry | Dismiss
```

## Principles

1. One job per state: deleting vs failed-cleanup.
2. No raw Postgres / AGE internals in the primary message.
3. Retry is first-class (indexes make retry succeed; message still honest if not).
4. Progress UI (SPEC-069) remains the happy-path surface; error toast/banner only on fail.

## Out of scope for visual redesign

New screens; only error copy + severity mapping on existing document lifecycle surfaces.
