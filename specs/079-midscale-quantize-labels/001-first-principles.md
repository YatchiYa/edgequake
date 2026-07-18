# SPEC-079 — First principles (mid-scale ≠ smoke)

Smoke (@2k) proves SQL shape and directionality. Mid-scale proves the tip still holds under **filtered** recall@20 when the corpus approaches the Supported Wave-2 floor (100k).

| Law | Application here |
|-----|------------------|
| Promote metric | Filtered recall@20 only |
| Cost | Bytes × candidates × (1 + I/O miss) — binary/labels shrink index RAM; mid-scale checks recall recovery |
| Honesty | Default stays Wave-2; tips stay opt-in OFF |
| Floors | Do not raise 100k/150k from tip arms without full concurrent gate |

## Arms

1. **B2** — `make`-equivalent of SPEC-077 with `EQ_BQ_ROWS∈{50k,100k}` on `pg18`
2. **A6** — SPEC-078 with `EQ_FDL_ROWS∈{50k,100k}` on `pg18-vectorscale`

Soft recall ≥0.90 vs Wave-2. Hang cliff via `EQ_BQ_HANG_MS` / `EQ_FDL_HANG_MS` (default 30s for mid-scale runner).

## Decision vocabulary

| Label | Meaning |
|-------|---------|
| **Not promoted** | Tip remains study-only (default expected outcome) |
| **promote candidate** | Filtered recall + latency sanity green @100k — still no silent flip; needs ops recipe before SSOT floor change |
