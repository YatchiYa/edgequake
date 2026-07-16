# Experiment Log — Accuracy Improvement

**Protocol:** one causal variable · query-only before re-ingest · full CORE ingest (`BENCH047_REQUIRE_FULL_INGEST=1`) · paired document-cluster bootstrap  
**Baseline B0:** CORE @40 Acc `0.4581` F1 `0.3564` · workspace `b994167c-180e-4708-96a1-a1778b450f15` · protocol `026-listmem-2026-07-15`

Use this template for every run. Copy a blank block; never overwrite prior decisions.

---

## Template

```text
ID:
Date:
Hypothesis:
Single changed variable:
Expected causal path (W1/W2/W3/W4):
Run type: query-only | fresh-ingest | extractor-only
Code SHA:
Workspace:
Artifact path:

Primary metric:
Guard metrics:
Paired Acc Δ + cluster 95% CI:
Paired F1 Δ + cluster 95% CI:
Slice wins/losses:
Latency/cost Δ:
Decision: promote | revise | revert
Notes:
```

---

## Queue

| ID | Change | Status | Decision |
|---|---|---|---|
| W0 | Failure ledger + paired bootstrap report | pending | — |
| G1 | Typed answer contract + normalizer | **next** | — |
| G2 | Deterministic grounded operations | blocked on G1 | — |
| G3 | List/set composer | blocked on G1 | — |
| G4 | Sufficiency shadow judge | pending | — |
| G5 | Selective answer/retry | blocked on G4 | — |
| R1–R5 | Retrieval waves | after M1 or G-wave plateau | — |
| D1–D4 | Representation waves | after retrieval | — |

---

## Completed

_(none yet)_

---

## Rejected / Reverted

| ID | Why |
|---|---|
| Acc #3 densify (historical) | Acc/F1 regression — do not revive |
| Medium vision-only (historical) | Chart a_in_e gate failed — not first lever |

---

## Promotion rules (copy)

- Promote only if primary paired CI lower bound > 0 **and** all guards pass.  
- Revert if F1 drops >0.01, UNA Acc drops >0.01, Chart/Table W1 fails, or gold metadata used at runtime.  
- Chart-8 Acc/F1 SOTA remains Acc #2 (`0.562`/`0.480`) until beaten on that fixture.
