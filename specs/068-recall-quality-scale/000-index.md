# SPEC-068 — Recall-quality scale + stack hygiene

**Status:** Complete (2026-07-18)  
**Depends on:** SPEC-067 (planner bias / ops floors), SPEC-066 (ceiling), SPEC-064 (Wave-2)  
**Goal:** Push mid-scale capacity past the HNSW recall cliff with first-principles + O(hot-set) engineering on Postgres 18 / pgvector ≥0.8.5 / AGE 1.8—without silent product-default flips or DiskANN.

**Outcome:** Mid-scale **honest wall** — no full-gate green above 100k; ops tip `ef_search=240` @100k only; AGE 1.8 + G1 green.

## Deliverables

| Item | Path |
|------|------|
| Index | this file |
| First principles | [`001-first-principles.md`](001-first-principles.md) |
| AGE entry-point audit | [`002-age-entry-point-audit.md`](002-age-entry-point-audit.md) |
| Artifacts | [`e2e/artifacts/`](e2e/artifacts/) |
| SSOT | [`docs/product-limits.md`](../../docs/product-limits.md) |

## Commands

```bash
# Recall × latency Pareto (Wave-2 + planner bias)
make recall-pareto
# Optional single N: EQ_PARETO_ROWS=150000 make recall-pareto

# AGE 1.8 G1 remasure
EQ_CEILING_STEP=G1 make ceiling-proof
# Archive G1 JSONL into specs/068-recall-quality-scale/e2e/artifacts/

make product-limits-check
```

## Locked decisions

- Promote only from full gate: Q1-d ∧ recall@20≥0.99 ∧ concurrent absolute
- No silent halfvec / global `ef_search` / `ef_construction` flip
- Ops tip recipe only after Pareto greens a rung
- DiskANN / pgvectorscale out of scope (no hang FORBIDDEN)
- AGE: pin bump + entry-point audit; keep native SQL expand hot path

## Checklist

- [x] Pack + first principles
- [x] AGE 1.8 pin + image verify; pgvector doc drift fixed
- [x] Recall×latency Pareto archived
- [x] Rebuild arm @250k (`m=32`/`ef_c=128`) — no promotion
- [x] Honest wall in SSOT + `ef_search=240` tip @100k only; `make product-limits-check` green
- [x] AGE entry-point audit + G1 on AGE 1.8 (~11.7 ms degrees)
