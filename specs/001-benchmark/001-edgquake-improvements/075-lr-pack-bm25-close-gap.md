# 075 — Close the LR gap: LR packing + BM25 (L1 after 074)

**Status:** L1 + L1.5 gates done · **labeled L2/Acc peer candidate** · **not** Acc Beat / not publish replace  
**Date:** 2026-07-22  
**Prior:** [074 Why EQ lags](./074-why-eq-lags-lightrag-medical-mid.md) · L0 [`T122410Z`](../e2e/artifacts/history/smoke-20260722T122410Z/)  
**L1:** [`T123021Z`](../e2e/artifacts/history/smoke-20260722T123021Z/) · **L1.5:** [`T123313Z`](../e2e/artifacts/history/smoke-20260722T123313Z/)  
**Warm:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5)  
**Ingest audit:** [`20260722T122714Z`](../e2e/artifacts/ingest-audit/20260722T122714Z/) — zero-chunk **0%**; EQ nodes ≥ LR; B6 ge2 still 0%

---

## 1. What L0 taught

| Metric | Acc headline Δ (medical-mid n=200) | L0 LR-identity Δ (smoke) |
|--------|-------------------------------------|---------------------------|
| ctx_rel | −0.095 | **−0.013** (PASS) |
| evidence_recall | −0.064 | −0.062 |
| Fact ER | EQ 0.790 / LR 0.953 | **EQ 0.700 / LR 0.900** (worse) |
| Acc CI | excludes 0 (LR ahead) | includes 0 (smoke only) |

**Binding:** Turning rerank **off** (fair LR pin) cleaned context but **dropped Fact gold** from Mix membership. Ingest provenance on B5 is already healthy (029 audit) — do **not** force-reingest as Acc headline.

**Revised L1 (query):** Keep BM25 (Acc Fact lexical) **and** LR packing identity (RR · bfs · VECTOR+LR budget · retrieval entity rank). One labeled pack vs Acc headline.

---

## 2. Pack `LR_PACK_BM25_v1`

| Pin | Acc headline | This pack |
|-----|--------------|-----------|
| `EDGEQUAKE_MIX_FUSION` | rrf | `round_robin` (+ `BENCH001_ALLOW_ROUND_ROBIN=1`) |
| `BENCH001_EQ_ENABLE_RERANK` | 1 | **1** (BM25 on) |
| `EDGEQUAKE_RERANKER` | bm25 | bm25 |
| `EDGEQUAKE_GRAPH_WALK` | ppr | `bfs` |
| `EDGEQUAKE_KG_CHUNK_PICK` | vector | vector |
| `EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET` | 0 | **1** |
| `EDGEQUAKE_ENTITY_RANK` | degree | `retrieval` |
| Dual-list / CE / path | off | off (no peer merge) |

```bash
make bench001-lr-pack-bm25          # L1: packing + global BM25
make bench001-lr-identity-fact-l2   # L1.5: L0 prompt + Fact L2 BM25 citations
```

### L1 result (`smoke-20260722T123021Z`)

| Gate | Result |
|------|--------|
| Fact ER 0.950 (≥ LR 0.900) | **PASS** |
| overall ER Δ −0.022 (≥ LR−0.03) | **PASS** |
| ctx_rel 0.419 (need ≥0.48) | **MISS** — BM25 re-noise (R3) |
| Acc CI | LR ahead — **no promote** |

**Tradeoff:** global BM25 restores Fact ER but undoes L0 ctx_rel. L1.5 keeps L0 Mix prompt + `L2_BM25_MODE=fact_replace` for Fact citations only.

### L1.5 result (`smoke-20260722T123313Z`) — keep labeled

| Metric | EQ | LR | Gate |
|--------|----|----|------|
| Acc | **0.806** | 0.772 | Δ+0.034 · CI includes 0 → **tie** (not Beat) |
| ctx_rel | **0.488** | 0.531 | ≥0.48 **PASS** |
| evidence_recall | **0.962** | 0.967 | ≥LR−0.03 **PASS** |
| Fact ER | **1.000** | 0.900 | **PASS** |

**Decision:** `LR_IDENTITY_FACT_L2_v1` smoke was promising; medical-mid n=200 peer ran ([`T125400Z`](../e2e/artifacts/history/medical-mid-20260722T125400Z/)).

### Medical-mid L1.5 peer (n=200)

| Metric | Acc headline | L1.5 peer | Gate |
|--------|--------------|-----------|------|
| Acc Δ CI | [−0.107, −0.033] LR | **[−0.061, +0.013] tie** | Acc Beat no; CI closed |
| ctx_rel | 0.396 | **0.474** | ≥0.50 **MISS** |
| ER | 0.887 | **0.946** | ≥LR−0.03 **PASS** |
| Fact ER | 0.790 | **0.919** | ≥LR−0.03 **MISS** |

Peer pack: `publish/peers/LR_IDENTITY_FACT_L2_v1/`. Acc `publish/latest` unchanged. Phase 4: [076](./076-mix-law-remaining-after-l15.md) naive-first RR.

```bash
make bench001-medical-mid-lr-identity-fact-l2   # n=200 labeled peer · SKIP publish/latest
make bench001-lr-nf-fact-l2                     # Phase4 smoke
make bench001-medical-mid-lr-nf-fact-l2         # Phase4 medical-mid peer
```

---

## 3. Gates (smoke)

| Gate | Target |
|------|--------|
| Honesty | No “EQ beats LightRAG”; Acc CI report-only |
| ctx_rel | ≥0.48 **or** Δ vs LR better than Acc headline (−0.095) |
| Fact ER | ≥0.80 **or** ≥ LR−0.10 (smoke); prefer ≥ LR−0.03 |
| overall ER | ≥ LR−0.03 preferred |
| Publish | medical-mid n=200 remains SSOT |

**Promote Acc headline only if** later medical-mid CI + L2 gates clear (not this smoke).

---

## 4. Why not dual-list / re-ingest next

- Dual-list (`a1lrl2`) needs CE; L0 had rerank off → Mix∪CE is a no-op.
- B5 zero-chunk already 0%; name coverage of LR 0.72 — Horizon B re-ingest is a **separate** workspace if Fact ER still misses after L1.
- B6 `ge2` relation multi-chunk (0% vs LR 11.9%) remains a structural follow-up, not this pack.

---

## 5. Cross-links

- [074](./074-why-eq-lags-lightrag-medical-mid.md) · [034 L2 dual-list](./034-l2-dual-list-under-full-ws-graph.md) · [029 ingest audit](./029-ingest-parity-audit.md) · [055 split peers](./055-post-acc-ceiling-first-principles.md)
