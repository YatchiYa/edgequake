# 048 — Summarize-only topic materialize (type-routed CE_GAP)

**Status:** **REJECT** Acc pin · Sum ER✓ · Fact ER/Acc tax via admit · keep B5+`a1fp`  
**Date:** 2026-07-20  
**Archive:** [`T132225Z`](../e2e/artifacts/history/smoke-20260720T132225Z/)  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Prior:** [045](./045-content-gated-materialize.md) · [047](./047-type-scoped-specificity.md)  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [043](./043-honesty-can-we-push.md)

---

## 1. Assess vs LightRAG (binding, post-047)

| Gap | EQ peer | LR | Law |
|-----|--------:|---:|-----|
| Acc | **0.801** | 0.782 | CI includes 0 → no Beat |
| Sum ER | ~0.86 | ~0.98 | **CE_GAP** |
| recall | 0.926 | 0.966 | Parity miss |
| Complex | gen gap | — | specificity Acc STOP |

**Confound:** type-route CONTENT materialize to Summarize only (`MATERIALIZE_TYPES=summarize`), keep admit for topic ids.

---

## 2. Acc results (`a1fpsumx` / T132225Z)

| Gate | Threshold | Result |
|------|-----------|--------|
| Acc | ≥ 0.781 | **FAIL 0.749** (Δ−0.040) |
| Fact ER | ≥ 0.83 | **FAIL 0.75** |
| ctx_rel | ≥ 0.50 | **PASS 0.50** |
| Sum ER | ≥ 0.95 or ≥LR−0.03 | **PASS 0.963** (LR 0.983) |
| bone phrase (Sum) | in context | **PASS** |

Pins: `topic_materialize_types=summarize` · content gate on · admit on.

---

## 3. Decision

```text
REJECT a1fpsumx as Acc pin
Sum ER law proven under Summarize-only mat
Fact ER tax = TOPIC_ENTITY_ADMIT (Exploratory) not mat types
TOPIC_* Acc fishing STOP (038–042, 045, 048)
keep B5+a1fp
next ≠ more TOPIC_* Acc pins; ≠ ANSWER_PROMPT Acc pins
next real ceiling = Horizon B ingest / dual-list L2 package (not Acc headline)
```
