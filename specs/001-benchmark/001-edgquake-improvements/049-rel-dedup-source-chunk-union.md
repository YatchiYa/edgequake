# 049 — Horizon B6: Relation dedupe source-chunk union

**Status:** **STRUCTURAL PROMOTE** · **Acc REJECT** · keep B5+`a1fp` peer  
**Date:** 2026-07-20  
**B6 WS:** `58ffe7da-d181-4a31-8941-9621b051a678`  
**Acc archive:** [`T140822Z`](../e2e/artifacts/history/smoke-20260720T140822Z/) Acc **0.725**  
**Ingest audit:** [`20260720T140630Z`](../e2e/artifacts/ingest-audit/20260720T140630Z/)  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801** · WS `8e990410-…`  
**Baseline (B5):** [`20260720T134941Z`](../e2e/artifacts/ingest-audit/20260720T134941Z/) EQ ge2 **0%**  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [044](./044-horizon-b-placeholder-provenance.md)

---

## 1. Assess vs LightRAG (no flaky heuristics)

| Gap | EQ B5 peer | LR | Binding |
|-----|-----------:|---:|---------|
| Acc | **0.801** | 0.782 | CI includes 0 |
| Sum ER / SELECT | ~0.86 | ~0.98 | TOPIC Acc STOP |
| Multi-chunk edges | **0%** | **11.9%** | **REL_DEDUP_SOURCE_LAST_WRITE** |

**Forbidden:** question_type / intent / bigrams / TOPIC_* Acc / answer-prompt Acc / soft Mix.

**Law:** collapsing `(src,tgt)` relations must **union** `source_chunk_ids` (entity + LightRAG `merge_source_ids` parity).

---

## 2. Results

| Gate | Result |
|------|--------|
| EQ edges ≥2 chunks rate ≥ 0.05 | **PASS 0.1247** (0 → 1046 edges; ≈ LR 0.1189) |
| mean chunks/edge | **1.182** (was 1.0; LR 1.17) |
| zero-chunk rate ≤ 0.01 | **PASS 0.0** |
| Acc ≥ 0.781 | **FAIL 0.725** (a1fp) |
| Fact ER ≥ 0.83 | **PASS 0.85** |
| ctx ≥ 0.50 | **PASS 0.506** |

Force-ingest smoke (pre-a1fp) Acc was 0.760 — still below peer.

---

## 3. Decision

```text
KEEP code (structural law closed — ship with product)
DO NOT promote B6 WS as Acc Fact peer
KEEP B5+a1fp Acc 0.801 (warm pointer restored)
next ≠ Acc fishing; optional B7 PLACEHOLDER_NO_VECTOR (325 AGE−VDB)
```

Reproduce:

```bash
make bench001-b6-reingest
# or query-only on B6 WS after ingest:
export BENCH001_EQ_WORKSPACE_ID=58ffe7da-d181-4a31-8941-9621b051a678
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp
```
