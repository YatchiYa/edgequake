# 046 — Answer specificity prompt (Complex Acc vs LightRAG)

**Status:** **REJECT** Acc pin · specificity law proven · keep B5+`a1fp`  
**Date:** 2026-07-20  
**Archive:** [`T124735Z`](../e2e/artifacts/history/smoke-20260720T124735Z/)  
**Peer keep:** [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [045](./045-content-gated-materialize.md)

---

## 1. Assess (binding)

Complex ER = **1.0** both sides → generation gap. EQ said “PARP inhibitor”; LR named drugs. Historic A3 `lightrag` abstain **hurt** Complex — skip. SELECT Acc fishing STOP (045).

---

## 2. Confound + results (`a1fpspec`)

`EDGEQUAKE_ANSWER_PROMPT=specific` on a1fp.

| Gate | Result |
|------|--------|
| Acc ≥ 0.781 | **FAIL 0.746** |
| Complex Δ vs LR ≤ 0.03 | **PASS −0.014** (was −0.05) |
| Fact ER ≥ 0.83 | **FAIL 0.80** |
| PARP drug names | **PASS** |

---

## 3. Decision

```text
specificity helps Complex Δ / named entities
Acc tax → do not promote
keep B5+a1fp
next ≠ TOPIC_* Acc fishing; ≠ A3 abstain re-run
```
