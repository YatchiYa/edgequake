# Lens 001 — Product Owner

## Stake

Concurrent email/document ingest is a partner-critical path. After 0.24.2, a bookkeeping unique index turns a previously silent race into **hard document failures**. That is a trust regression.

## Outcome

| Priority | Outcome |
|----------|---------|
| P0 | No user-visible GraphMerge from `idx_*_legacy_vector_id` |
| P0 | Concurrent same-WS ingest succeeds (absorb loser stamp) |
| P1 | Stable resolve ordering (less dual-FK churn) |
| Later | Alias entity completeness (SPEC-083) — tracked, not blocking |

## Acceptance language

> “Two documents that introduce the same entity name at the same time finish processing. Operators do not need to lower concurrency or manually reprocess for this error.”

## Non-goals

- Marketing UI / new screens
- Promising zero duplicate entity rows historically (backfill is separate)
