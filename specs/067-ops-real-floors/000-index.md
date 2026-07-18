# SPEC-067 — Ops-real floors + accessible limits docs

**Status:** Complete (2026-07-18)  
**Depends on:** SPEC-066 (ceiling artifacts), SPEC-065 (SSOT), SPEC-064 (Wave-2)  
**Goal:** Make Wave-2 floors ops-real (planner bias, admission/readiness) and rewrite capacity docs operator-first—without inventing unproven capacity claims.

## Deliverables

| Item | Path |
|------|------|
| Index | this file |
| Artifacts | [`e2e/artifacts/`](e2e/artifacts/) |
| SSOT | [`docs/product-limits.md`](../../docs/product-limits.md) |

## Commands

```bash
# Remasure SEEK after planner bias
EQ_CEILING_STEP=SEEK EDGEQUAKE_CEILING_ROWS=250000 make ceiling-proof
# Copy JSONL into specs/067-ops-real-floors/e2e/artifacts/
make product-limits-check
```

## Locked decisions

- Session-local planner bias only (no global HNSW drop in prod)
- No silent halfvec / GUC default flip
- Promote SSOT only from green remasure artifacts
- DiskANN still out of scope

## Checklist

- [x] Wave-2 planner bias + unit/EXPLAIN
- [x] max_documents harden + e2e 409
- [x] `/ready` fail-closed on Wave-2 probe Err
- [x] Operator-first product-limits + FAQ/perf-tuning/.env
- [x] SEEK 250k remasure archived (`uses_partial=true`; concurrent green; recall cliff → no SSOT floor raise)
