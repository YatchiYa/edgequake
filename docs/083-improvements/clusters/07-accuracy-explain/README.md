# Cluster 07 — Accuracy, evaluation, explainability

> **Sprint**: 5  
> **Laws**: LAW-3, LAW-8  
> **Defects**: X-21, X-34, X-35, D-54 (+ Wave D: X-17, D-32 in identity cluster)

---

## WHY

Marketed accuracy numbers come from tiny corpora (@5 docs). Full-scale Acc falls monotonically to ~0.458@40. Golden fixtures used to be **counted**, not scored (X-34). **ExplainTrace MVP is FIXED** (X-21). Community detection was flat Louvain phase-1 only (D-54).

## ROOT CAUSE

```
  Acc@5 0.549 --> @40 0.458   (systemic, not one line)

  Contributing defects (fix upstream first):
    D-38 query_vec history pollution   [FIXED]
    D-30 multigraph collapse           [FIXED]
    C-14 normalization dupes           [FIXED]
    D-35/36/37/39 fusion & thresholds  [mixed; see register]
    D-54 no community hierarchy        [FIXED — opt-in flag]
    X-34 decorative golden             [FIXED — deterministic score gate]
    X-21 ExplainTrace                  [FIXED]
    X-35 Acc@N regression              [FIXED — floors JSON gate]
```

Accuracy remains an **outcome metric**. Wave D lands measurement gates + optional hierarchy/fuzzy; it does **not** magically raise Acc@40.

## SOLUTION (Wave D landed)

| Item | Status | How |
|------|--------|-----|
| Acc@N / F1@N honesty | Done | `acc_at_n_floors.json` + `bench_acc_at_n_regression_gate` |
| Golden Acc gate (X-34) | Done | `nightly_golden_acc_gate` scores every case (mock oracle keyword Acc/F1); live LLM path `#[ignore]` |
| ExplainTrace (X-21) | Done | `ExplainTrace` / API DTO |
| Louvain hierarchy (D-54) | Done | `EDGEQUAKE_LOUVAIN_HIERARCHY=1` → phase-2 aggregation; `unit_louvain_hierarchy_levels` |
| Fuzzy entity resolve (X-17) | Done | `EDGEQUAKE_ENTITY_FUZZY=1` (default off); blocking + Levenshtein/Jaccard |
| Entity type conflict (D-32) | Done | majority/confidence votes + conflict logs; `e2e_entity_type_conflict_logged_and_resolved` |

## EDGE CASES

Domain shift (medical vs general); LLM-as-judge cost; flaky judges → deterministic metrics in CI. Fuzzy matching can over-merge short names — threshold tunable via `EDGEQUAKE_ENTITY_FUZZY_THRESHOLD`. Tiny graphs: hierarchy may stop after one level.

## E2E

`nightly_golden_acc_gate`, `bench_acc_at_n_regression_gate`, `unit_louvain_hierarchy_levels`, ExplainTrace API contract, `e2e_entity_type_conflict_logged_and_resolved`, `contract_x_17` / `e2e_x_17`
