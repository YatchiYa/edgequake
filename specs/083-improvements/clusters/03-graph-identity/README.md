# Cluster 03 — Graph identity & merge semantics

> **Sprint**: 2 (identity), 3 (merge gates), 4 (fuzzy)  
> **Laws**: LAW-6, LAW-7, LAW-3  
> **Defects**: C-14 FIXED · D-30/D-31/D-33/D-34 FIXED · X-15 FIXED · D-32 FIXED · X-17 FIXED (opt-in) · C-26 CONFIRMED

---

## WHY

Duplicate entities, collapsed relation types, unstable weights, and truncated lineage historically made the KG lie. Structural fixes for normalize, multigraph, weight policy, lineage-before-cap, token gate SSOT, and OTHER type are **FIXED**. Wave D also lands **D-32** type majority/confidence voting and optional **X-17** fuzzy resolution.

## ROOT CAUSE (historical map)

```
  normalize bugs --> THE COMPANY ≠ The Company     [FIXED C-14]
  edge unique without type --> type collapse       [FIXED D-30]
  weight (a+b)/2 non-associative                   [FIXED D-31]
  cap source_ids THEN lineage                      [FIXED D-33]
  NeedsLlm@1200 vs summarizer@4000                 [FIXED D-34]
  entity_type first-wins                           [FIXED D-32]
  exact-match entity resolution                    [FIXED X-17 opt-in]
```

## SOLUTION

| Concern | SSOT | Status |
|---------|------|--------|
| Names | `normalize_entity_name` | FIXED |
| Edges | Unique `(src,tgt,rel_type)` | FIXED |
| Weight | `WeightPolicy::{Max, MeanCounted}` | FIXED |
| Lineage | Document set before cap | FIXED |
| Description gate | Single 1200 threshold | FIXED |
| OTHER | Default entity types | FIXED |
| Type conflicts | `entity_type_vote` majority/confidence + logs | FIXED |
| Fuzzy resolve | `EDGEQUAKE_ENTITY_FUZZY=1` (default off) | FIXED opt-in |
| Caps | MAX_SOURCE_IDS unused (C-26) | CONFIRMED backlog |

## E2E

`unit_normalize_*`, `e2e_multigraph_two_rel_types_persist`, `unit_weight_associative`, `e2e_lineage_includes_docs_beyond_source_cap`, `unit_needs_llm_always_summarizes`, `contract_other_in_default_entity_types`, `e2e_entity_type_conflict_logged_and_resolved`, `contract_x_17` / `e2e_x_17`  
Backlog: `e2e_merge_duplicate_nodes_migration`
