# 13 — Lens: AI Engineer

## Jobs vs schema

Confirm-drop is **not** an embedding job. If 126/131 abort, the model fleet
already wrote vectors into **legacy** `eq_*_vectors`. Typed SSOT needs:

| Job | Why |
|-----|-----|
| `w3-chunk-embedding-backfill` | `{doc}-chunk-{n}` → `chunk_embeddings` |
| `iw2-fleet-embedding-backfill` | entity / relationship / report rows |
| `iw2-fleet-provenance-stamp` | set `legacy_vector_id` without re-embedding |

`EDGEQUAKE_MIGRATION_MODE=automatic` runs engine jobs; `verify` checks coverage.
`EDGEQUAKE_MIGRATION_VERIFY_EQUALITY=0` when embeddings were **regenerated**
(bit inequality with same coverage) — SPEC-111.

## Fail-closed persist

Do not “fix” migrate by skipping provenance. Exact-name fallback across
workspaces was removed (false GREEN). Stamp or rewrite typed rows.

## Not in scope for 137

Prompt cache (SPEC-126), pack-to-budget (SPEC-135), reasoning effort (SPEC-109).
Those do not gate 125/126/131.
