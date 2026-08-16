# 11 — Honest assessment

## What this fixes

A real CHECK violation on a dual-write path that operators see as WARN spam and stale list status during slim-checkpoint resume.

## What this does not fix

- **#377** persistent `idx_entity_embeddings_legacy_vector_id` / relationship twin collisions.
- Documents that end “indexed” with 0 entity vectors after merger partial failure.
- Embedding dimension mismatch errors reported on the same fleets.

Those can still create crash checkpoints that **exercise** the resume path; after SPEC-129 they should no longer trip `documents_valid_status` on `re_embedding`.

## Residual risk

| Risk | Level | Notes |
|------|-------|-------|
| Future writer forgets helper | Med | Contract grep + code review |
| FE assumes column == rich stage | Low | Documented display≠column |
| Default `_` → `processing` hides typos | Low | Prefer logging unknown stages later (non-goal) |

## Confidence

High on root cause (code + log timing). High on fix correctness (reuse proven shell map). Medium on fleet silence until soak without #377 WARNs mixed in.

## Cross-refs

- Why: [00-why.md](00-why.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
