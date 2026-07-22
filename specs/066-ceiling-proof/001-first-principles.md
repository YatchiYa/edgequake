# SPEC-066 — First principles

## Law

Capacity claims require: **(1)** hard cap, **(2)** physics + host class, or **(3)** measured gate.  
A completed ladder with `slo_pass=false` under hang cliff is a **measured cliff**, not support.

## Cost model (from SPEC-064)

```
cost ≈ embedding_bytes × filtered_heap_rows × (1 + I/O_miss_penalty)
```

| Lever | Effect |
|-------|--------|
| halfvec | ~0.5× embedding bytes → residency win |
| workspace partial HNSW | ANN over hot WS slice (not btree→exact on 20%) |
| shared_buffers / RAM | removes I/O miss penalty (cold cliff) |

## Ceiling method

1. Fix Wave-2 shape + residency (2–4 GB `shared_buffers`).
2. Measure L2 (500k) → L3 (1M) → seek steps between last green and first fail.
3. Record `highest_green_N` and `first_fail_N` with EXPLAIN + host RAM.
4. Promote SSOT only for green rungs.

## Separate axes

- **Vectors** ≠ **documents** ≠ **entities** — do not promote FAQ rows across axes.
