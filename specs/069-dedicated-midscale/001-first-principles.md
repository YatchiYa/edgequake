# SPEC-069 — First principles

## Problem (from SPEC-068)

At **150k** under shared+partial Wave-2: ANN-relative recall@20 = **1.00**, but concurrent p95 ~650–675 ms (**fails** absolute Q1-d). Binding cliff is **contention / hot-set shape**, not quality.

## Principles

1. **O(hot-set)** — Dedicated per-workspace tables make table size = workspace size; skip partial HNSW (`is_dedicated_workspace_table`).
2. **Prove the production isolation path** — `PgWorkspaceVectorRegistry` namespaces `*_ws_*`; ceiling/Pareto only proved shared+partial.
3. **Do not redefine the gate** — Promote only if clients=**16** absolute Q1-d + recall@20≥0.99 + single Q1-d.
4. **DiskANN is last resort** — Open SPEC-070 only if dedicated 150k still fails after residency + ef tip + scan_mem tip.
5. **Evidence before claims** — No SSOT raise without SPEC-069 JSONL.

## Promote law

```
rung_green ⇔ single_p95 < 500ms ∧ recall@20_ann ≥ 0.99 ∧ concurrent_abs_p95 < 500ms @ clients=16
```
