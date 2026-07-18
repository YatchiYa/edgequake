# SPEC-080 RUN_NOTES — Tiny-slice exact

- Date: 2026-07-18
- Env: `EDGEQUAKE_ANN_EXACT_MAX_ROWS` (default 2000)
- Wave-2 planner bias skipped when workspace rows ≤ threshold
- Floors unchanged; no silent flip of Wave-2 defaults
- Contract exit: 0
- Lib/filter exit: 0
- Bias unit exit: 0
- DB smoke: skipped (contracts cover bias skip; EQ_TINY_SLICE_SMOKE=1 for DB)

## Gate: GREEN

