# 077 — Dense-only Mix arms on L1.5 (close ctx / Fact ER)

**Status:** E1/E2 ladder · **not** Acc Beat / not Acc headline  
**Date:** 2026-07-22  
**Keep base:** L1.5 [`T125400Z`](../e2e/artifacts/history/medical-mid-20260722T125400Z/) — Acc CI tie · ctx 0.474 · Fact ER 0.919  
**Prior:** [076](./076-mix-law-remaining-after-l15.md) · [075](./075-lr-pack-bm25-close-gap.md)

---

## 1. Binding gaps (first principles)

L1.5 already: RR · rerank off · VECTOR+LR budget · Fact L2 `fact_replace`. Acc CI closed to a **tie**. Still open:

| Gap | L1.5 mid | Target |
|-----|----------|--------|
| ctx_rel | 0.474 | ≥0.50 or ≥0.494 (+0.02) |
| Fact ER | 0.919 | ≥ LR − 0.03 |
| Creative ctx | 0.305 | raise without Acc CI tax |

**Law:** Fair LR Mix arms are dense-only when rerank is off. EQ still fuses in-arm BM25 (`EDGEQUAKE_BM25_RETRIEVAL` default on) into the Acc prompt → residual noise (076 R2).

NF naive-first (R1) Acc CI **REJECT** on mid — do not retry.

---

## 2. E1 — Dense-only arms

| Pin | L1.5 | E1 |
|-----|------|-----|
| L1.5 pack | on | on |
| `EDGEQUAKE_BM25_RETRIEVAL` | 1 (default) | **0** |
| `RR_ORDER` | local_first | local_first |

```bash
make bench001-lr-dense-fact-l2              # smoke
make bench001-medical-mid-lr-dense-fact-l2  # n=200 peer
```

Profile: `LR_DENSE_FACT_L2_v1`. Peer: `publish/peers/LR_DENSE_FACT_L2_v1/`.

**Gates:** Acc CI includes 0 or ci_low ≥ −0.061 · ctx ≥0.50 or ≥0.494 · ER ≥LR−0.03 · no Beat · Acc `publish/latest` intact.

**Stop:** Acc CI worse than L1.5 → REJECT E1 → E2 on L1.5 base.

### E1 results

| Stage | Acc CI | ctx_rel | Fact ER | Verdict |
|-------|--------|---------|---------|---------|
| Smoke [`T131449Z`](../e2e/artifacts/history/smoke-20260722T131449Z/) | tie [−0.044, +0.123] | **0.525** | 0.95 | green |
| Mid [`T132147Z`](../e2e/artifacts/history/medical-mid-20260722T132147Z/) | **[−0.083, −0.010] LR** (ci_low < L1.5 −0.061) | **0.504** | **0.943** | **REJECT Acc** |

Dense arms lift L2 (ctx/Fact ER) but Acc CI regresses vs keep L1.5 → do not keep E1. Acc `publish/latest` untouched (P0).

---

## 3. E2 — Occurrence sort (if Fact ER still open)

On keep base (E1 if keep, else L1.5): `EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT=1`.

```bash
make bench001-lr-occ-fact-l2
make bench001-medical-mid-lr-occ-fact-l2
```

Profile: `LR_OCC_FACT_L2_v1`. Success: Fact ER ≥LR−0.03 ∧ Acc CI not worse than keep base.

### E2 results (base = L1.5; E1 rejected)

| Stage | Acc CI | ctx_rel | Fact ER | Verdict |
|-------|--------|---------|---------|---------|
| Smoke [`T132404Z`](../e2e/artifacts/history/smoke-20260722T132404Z/) | tie [−0.054, +0.091] | **0.563** | 0.95 | green |
| Mid [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/) | **[−0.031, +0.040] tie** (best) | 0.491 | **0.917** (< LR−0.03) | **KEEP Acc CI** · Fact ER still open |

Fact ER gate **miss** → stop Acc packing fishing. Keep peer = E2 (best Acc CI among L1.5/E1/E2). Acc `publish/latest` untouched (P0).

| Pack | Acc CI | ctx | Fact ER | Role |
|------|--------|-----|---------|------|
| L1.5 | [−0.061, +0.013] | 0.474 | 0.919 | prior keep |
| E1 dense | [−0.083, −0.010] LR | 0.504 | 0.943 | REJECT Acc |
| **E2 occ** | **[−0.031, +0.040]** | 0.491 | 0.917 | **keep** |

---

## 4. Explicitly not

- Acc headline promote / silent `publish/latest`
- Soft Mix Acc fishing / CE dual-list stack
- Re-running NF RR / stacking more packing confounds after E2 Fact ER miss
