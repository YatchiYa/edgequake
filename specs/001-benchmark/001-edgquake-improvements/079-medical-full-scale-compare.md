# 079 — Medical-full scale compare (n≈2062)

**Status:** E2 + P0 full done · **not** Acc Beat / not Acc SSOT  
**Date:** 2026-07-22/23  
**Keep mid peer:** E2 occ [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**E2 full peer:** [`T171906Z`](../e2e/artifacts/history/medical-full-20260722T171906Z/) · `publish/peers/LR_OCC_FACT_L2_medical_full_v1`  
**P0 full peer:** [`T204100Z`](../e2e/artifacts/history/medical-full-20260722T204100Z/) · `publish/peers/P0_MEDICAL_FULL_v1`  
**Acc SSOT:** P0 medical-mid n=200 [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) · `publish/latest`  
**Prior:** [078](./078-eq-vs-lightrag-first-principles-next.md)

---

## 1. Why

medical-mid n=200 Acc CI is underpowered for fine gaps. **medical-full** = all GraphRAG-Bench medical questions (n=2062) on the **same warm corpus** as mid (query-only B5 WS). Stronger Acc CI; no novel confound (defer `core`).

```bash
make bench001-medical-full-lr-occ-fact-l2   # E2 keep pack peer
make bench001-medical-full-p0              # P0 pack peer (skip publish/latest)
```

---

## 2. Mid vs full

| Pack / stage | n | EQ Acc | LR Acc | Acc Δ CI | ctx | Fact ER |
|--------------|---|--------|--------|----------|-----|---------|
| E2 mid | 200 | 0.765 | 0.760 | [−0.031, +0.040] tie | 0.491 | 0.917 |
| E2 full | 2062 | 0.739 | 0.784 | [−0.069, −0.017] **LR** | 0.472 | 0.918 |
| P0 mid (Acc SSOT) | 200 | 0.706 | 0.774 | [−0.107, −0.033] LR | 0.396 | 0.790 |
| P0 full | 2062 | 0.724 | 0.784 | [−0.107, −0.042] **LR** | 0.394 | 0.905 |

### Reading

- Mid E2 Acc **tie** does **not** hold at n=2062 — LR ahead with CI excluding 0.
- P0 full confirms Acc SSOT direction: LR ahead; CI still excludes 0 (slightly wider low than mid).
- E2 full still beats P0 full on EQ Acc (0.739 vs 0.724) and ctx (0.472 vs 0.394) — labeled gap-close keep remains E2 mid for CI-tie claims; full-N is scale check only.
- Acc `publish/latest` stays mid n=200.
- No Beat: CI excludes 0 for EQ fails; ctx &lt; 0.50 on both full packs.

---

## 3. Cost / ops

- E2 full wall ≈1h45; P0 full wall ≈3h (judge 429 bursts during retrieval scoring; run still `valid=true`).
- Warm indexes reused (`index_stage=smoke`).
- OrbStack VM was briefly Stopped after earlier Acc pool timeout; recovered with `orb start` before P0 relaunch.
