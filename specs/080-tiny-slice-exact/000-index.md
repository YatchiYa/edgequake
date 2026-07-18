# SPEC-080 — B3 tiny-slice exact (planner honesty)

**Status:** Complete  
**Depends on:** SPEC-067 (Wave-2 planner bias), SPEC-073 §006 B3  
**Goal:** Skip Wave-2 `enable_seqscan=off` bias when the filtered workspace row count ≤ `EDGEQUAKE_ANN_EXACT_MAX_ROWS` (default **2000**) so Postgres/pgvector can prefer exact search on tiny slices.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default threshold | 2000 rows |
| Env | `EDGEQUAKE_ANN_EXACT_MAX_ROWS` |
| Floors | Unchanged |
| Silent flip | N/A (removes over-bias; does not force seqscan) |

## Commands

```bash
cargo test -p edgequake-storage --features postgres --test contract_spec080_tiny_slice_exact
make tiny-slice-exact-gate
make product-limits-check
```

## Checklist

- [x] Policy + bias skip wired
- [x] Contract + gate
- [x] SSOT tip; SPEC-073 B3 linked
