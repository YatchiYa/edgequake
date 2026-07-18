# SPEC-064 — Lever matrix

| Arm | Wave | Storage | Index | Env |
|-----|------|---------|-------|-----|
| `full_default` | 0 | `vector` | global HNSW | (baseline) |
| `halfvec_default` | 1 | `halfvec` | global HNSW | `EDGEQUAKE_VECTOR_STORAGE=halfvec` (or `with_storage_mode`) |
| `halfvec_partial_ws` | 2 | `halfvec` | partial HNSW on `ws-a` | `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1` |
| `guc_grid` | 3 | winner | winner | `EDGEQUAKE_HNSW_EF_SEARCH`, `EDGEQUAKE_HNSW_MAX_SCAN_TUPLES`, `EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER` |

## Gates per arm

| Metric | Op name pattern | Pass |
|--------|-----------------|------|
| Single | `battle_<arm>_single` | p95 &lt; 500ms **or** (Wave1) ≥2× vs Wave0 **and** recall OK |
| Stress | `battle_<arm>_stress` | ≤1.5× single (pg18) |
| Recall | `battle_<arm>_recall` | @20 mean ≥ 0.99 vs full |
| EXPLAIN | `battle_<arm>_explain` | No Seq Scan; Wave2 must name partial index |
| Gate | `battle_gate_summary` | any arm `slo_pass` for envelope promote |

## Promote rules

1. Do **not** flip prod `EDGEQUAKE_VECTOR_STORAGE` until battle green + data-layer checklist.
2. Partial indexes remain **opt-in** (`EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1`) for hot workspaces.
3. Wire GUC defaults only when Wave3 knee is green under recall.
4. Update [`../063-architecture-capacity-assessment/003-operating-envelope.md`](../063-architecture-capacity-assessment/003-operating-envelope.md) only when L1 `slo_pass=true`.
