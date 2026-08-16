# Lens 001 — Product Owner

## Pain

Operators reprocess failed documents and still see **Failed** (or stale status) on the Documents list while logs show active re-embedding. WARNs erode trust in “non-fatal” dual-write.

## Outcome

- Resume / reprocess shows a live processing-class status on SQL-backed list columns.
- No recurring `documents_valid_status` WARN on healthy resumes.
- #377 remains a separate priority; this ship unblocks status freshness independently.

## Acceptance (PO)

1. Reprocess with slim checkpoint does not emit #381 WARN.
2. Documents list leaves prior `failed` once resume starts (column → `processing` while KV may say `re_embedding`).
3. No schema migration required for fleet upgrade.

## Non-goals

- Fixing entity embedding collisions (#377).
- New user-facing status enum values in SQL.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
