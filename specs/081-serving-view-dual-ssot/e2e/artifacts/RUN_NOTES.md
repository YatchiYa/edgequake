# SPEC-081 RUN_NOTES — Serving view dual-SSOT

- Date: 2026-07-18
- Functions: `eq_serving_chunk_presence(uuid)`, `eq_serving_vector_presence(uuid, regclass)`
- Serving view ≠ RAG ANN SSOT; ingest/ANN paths unchanged
- Silent store unify: forbidden
- Contract exit: 0
- DB probe: probed exit=0

## Gate: GREEN

## Phase-4 note

Broader dual-SSOT narrowing only if retract surfaces decrease without recall loss.
