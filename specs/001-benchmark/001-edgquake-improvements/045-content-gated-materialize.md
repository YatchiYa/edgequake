# 045 — CONTENT-gated topic materialize (First Principles vs LightRAG)

**Status:** **REJECT** Acc pin · SELECT ladder **STOP**  
**Date:** 2026-07-20  
**Archive:** [`T121724Z`](../e2e/artifacts/history/smoke-20260720T121724Z/)  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Cross-ref:** [043](./043-honesty-can-we-push.md) · [042](./042-topic-chunk-materialize.md) · [044](./044-horizon-b-placeholder-provenance.md) · [028](./028-first-principles-beat-roadmap.md)

---

## 1. Assess EQ vs LightRAG (B5+a1fp peer)

| Dimension | EQ | LR | Binding? |
|-----------|---:|---:|:--------:|
| Acc | **0.801** | 0.782 | Point win; CI includes 0 → **no Beat** |
| Fact Acc | **0.765** | 0.685 | EQ ahead |
| Complex Acc | 0.813 | **0.863** | Δ−0.05 |
| Sum ER | **0.863** | **0.983** | **−0.12 — main L2 hole** |
| Overall recall | 0.926 | 0.966 | need ≥0.936 for Parity |
| ctx | 0.519 | 0.519 | tie ✓ |

---

## 2. Confound + results (`a1fpcmat`)

`TOPIC_MATERIALIZE_CONTENT=1`: scan admit pool, inject ≤4 KV bodies with question content bigrams.

| Gate | Result |
|------|--------|
| Acc ≥ 0.755 | **FAIL 0.733** |
| Fact ER ≥ 0.83 | **FAIL 0.80** |
| Sum ER ≥ 0.90 | **PASS 0.963** |
| Probe `bone cancers` | **PASS** |
| vs peer Acc tax | **REJECT** (−0.068) |

---

## 3. Decision

```text
CONTENT gate closed CE_GAP for Sum/probe
Acc/Fact tax → do not promote
STOP topic-SELECT Acc fishing
keep B5+a1fp peer
next ≠ another TOPIC_* Acc pin
```

Env (default off): `EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT`. Ladder: `a1fpcmat`.
