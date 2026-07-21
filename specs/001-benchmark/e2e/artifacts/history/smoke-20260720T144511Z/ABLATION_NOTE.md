# Ablation — B7 PLACEHOLDER_VDB_PARITY + a1fp

**Step:** B7 re-ingest → a1fp  
**Pins:** 050 B7: AGE placeholders get entities_vdb (`{name}\n{relation_description}`) + a1fp query pack  
**Workspace:** `dbaf36a1-6a59-4d3d-9438-8a84da92bdc9`  
**Archive:** `smoke-20260720T144511Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| age_over_vectors | ∈ [0.98, 1.02] | **1.0** ✓ |
| eq_zero_chunk_rate | ≤ 0.01 | **0.0** ✓ |
| Acc | ≥ 0.781 (peer ≥ 0.801) | **0.676** ✗ |
| Fact ER | ≥ 0.83 | **0.80** ✗ |
| ctx_rel | ≥ 0.50 | **0.506** ✓ |
| Δ Acc 95% CI | Beat excludes 0 EQ | includes 0 (Δ −0.093) |

## Verdict

- [x] Structural gate met (PLACEHOLDER_VDB_PARITY closed)
- [x] Acc gate missed — **do not promote**; keep B5+`a1fp` Acc peer `8e990410-…` / T120315Z 0.801
- Keep B7 merger code (law, not Acc lever)
