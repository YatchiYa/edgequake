# SPEC-082 — Push-scale performance + floor attempt

**Status:** Complete  
**Depends on:** SPEC-072/079/078, SPEC-068, SPEC-075  
**Goal:** Push measured N further — A6 @150k/250k, Wave-2 filtered spot @150k, DiskANN **primary full-gate @250k**. Raise SSOT floors **only** when full-gate green; otherwise archive **Not promoted**.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Wave-2 default | Remains **100k** (SPEC-082 single-spot @150k green ≠ concurrent floor) |
| DiskANN opt-in | **`highest_green_N=250000`** — full-gate green @250k (list≥800, HQ build) |
| A6 tip | Soft-green @150k (0.95); soft-fail @250k (0.05) — still not product default |
| Silent flip | Forbidden |
| Promote metric | Filtered recall@20 (A6/Wave-2); DiskANN dedicated full-gate (recall∧single∧concurrent) |

## Commands

```bash
make push-scale-ladder
# EQ_PUSH_A6_ROWS=150000,250000 EQ_PUSH_WAVE2_ROWS=150000 EQ_PUSH_DISKANN=1 make push-scale-ladder

make product-limits-check
```

## Checklist

- [x] Pack + first principles
- [x] Runner + artifacts (`e2e/artifacts/RUN_NOTES.md`)
- [x] SSOT DiskANN opt-in floor → **250k** (Wave-2 unchanged)
