# 079 — Medical-full scale compare (n≈2062)

**Status:** Acc-law 086 full done · **not** Acc Beat / not Acc SSOT  
**Date:** 2026-08-16 (refresh) · Jul-22 E2/P0 ladder retained  
**Acc SSOT:** E2-occ 086 medical-mid n=200 [`T110218Z`](../e2e/artifacts/history/medical-mid-20260815T110218Z/) · `publish/latest` (EQ Acc **0.792** / LR **0.786** tie)  
**Acc-law full peer:** [`T012004Z`](../e2e/artifacts/history/medical-full-20260816T012004Z/) · `publish/peers/ACC_E2OCC_086_MEDICAL_FULL_v1` (EQ Acc **0.786** / LR **0.786** point tie · chunk **1200/100**)  
**Keep mid peer (gap-close CI):** E2 occ [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**E2 full peer:** [`T171906Z`](../e2e/artifacts/history/medical-full-20260722T171906Z/) · `publish/peers/LR_OCC_FACT_L2_medical_full_v1`  
**P0 full peer:** [`T204100Z`](../e2e/artifacts/history/medical-full-20260722T204100Z/) · `publish/peers/P0_MEDICAL_FULL_v1`  
**Prior:** [078](./078-eq-vs-lightrag-first-principles-next.md) · Beat program [088](./088-beat-ctx-fact-er-program.md)

---

## 1. Why

medical-mid n=200 Acc CI is underpowered for fine gaps. **medical-full** = all GraphRAG-Bench medical questions (n=2062) on the **same warm corpus** as mid (query-only B5 WS). Stronger Acc CI; no novel confound (defer `core`).

```bash
make bench001-medical-full-lr-occ-fact-l2   # E2 keep pack peer
make bench001-medical-full-p0              # P0 pack peer (skip publish/latest)

# Acc-law 086 full (chunk 1200/100, query-only, skip publish/latest):
export BENCH001_EQ_WORKSPACE_ID=23b09c73-aa3f-4497-8e11-c448ffad8c53
export BENCH001_QUERY_ONLY=1 BENCH001_SKIP_PUBLISH_LATEST=1
export BENCH001_PUBLISH_PEER=ACC_E2OCC_086_MEDICAL_FULL_v1
# default BENCH001_EQ_CHUNK_SIZE=1200
python3 -m bench001.cli medical-full --api http://127.0.0.1:8090 --query-only \
  --profile-id ACC_E2OCC_086_v1 --query-concurrency 4 --eval-concurrency 24
```

---

## 2. Mid vs full

| Pack / stage | n | EQ Acc | LR Acc | Acc Δ CI | ctx | Fact ER |
|--------------|---|--------|--------|----------|-----|---------|
| Acc-law 086 mid (Acc SSOT) | 200 | 0.792 | 0.786 | [−0.022, +0.034] tie | 0.471 | 0.847 |
| Acc-law 086 full | 2062 | **0.786** | **0.786** | [−0.160, +0.047] tie (paired n=16) | 0.427 | 0.914 |
| E2 mid | 200 | 0.765 | 0.760 | [−0.031, +0.040] tie | 0.491 | 0.917 |
| E2 full | 2062 | 0.739 | 0.784 | [−0.069, −0.017] **LR** | 0.472 | 0.918 |
| P0 mid (Jul-22 Acc SSOT) | 200 | 0.706 | 0.774 | [−0.107, −0.033] LR | 0.396 | 0.790 |
| P0 full | 2062 | 0.724 | 0.784 | [−0.107, −0.042] **LR** | 0.394 | 0.905 |

### Reading

- **Acc-law 086 full (2026-08-16):** Acc **point tie** 0.786/0.786 — closed the P0-full 6pp scale gap (0.724 → 0.786). **Not Beat:** ctx 0.427 &lt; 0.50; Acc CI includes 0 (bootstrap paired n=16 underpowered). Acc `publish/latest` stays mid n=200 [`T110218Z`](../e2e/artifacts/history/medical-mid-20260815T110218Z/).
- Jul-22 mid E2 Acc **tie** did **not** hold at n=2062 — LR ahead with CI excluding 0 (that scale gap is closed under Acc-law 086).
- P0 full was the old scale direction: LR ahead. Acc-law 086 full supersedes it as the labeled scale check.
- E2 full still beats P0 full on EQ Acc (0.739 vs 0.724) and ctx (0.472 vs 0.394) — historical gap-close keep remains E2 mid for CI-tie claims.
- No Beat: ctx &lt; 0.50 on every full pack. Acc ingest pin remains **chunk 1200/100**.

---

## 3. Cost / ops

- Acc-law 086 full (`T012004Z`) is query-only on warm ws `23b09c73-…` (no re-ingest). Raw JSON is local-only (gitignore; &gt;100MB).
- E2 full wall ≈1h45; P0 full wall ≈3h (judge 429 bursts during retrieval scoring; run still `valid=true`).
- Warm indexes reused (`index_stage=smoke`).
- OrbStack VM was briefly Stopped after earlier Acc pool timeout; recovered with `orb start` before P0 relaunch.
