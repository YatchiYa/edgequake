# 084 — Honest Equal LightRAG Rebench (No Flaky Acc Fishing)

**Status:** H3 **REJECT Parity** (ctx tax) · Acc CI improved labeled · Acc Beat **STOP** · H4 skipped · successor **[085](./085-fairness-concurrency-equal-stop.md)** (fair mid · Acc Equal STOP)  
**H3 archive:** [`T041648Z`](../e2e/artifacts/history/medical-mid-20260723T041648Z/) · peer `E2_CHAT_SPLIT_v1`  
**Date:** 2026-07-23  
**Parent:** [083](./083-lightrag-query-api-law.md) · [082](./082-gold-citation-compat.md) · [081](./081-beat-parity-first-principles.md)  
**Warm WS:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5)  
**Keep query base:** E2 occ [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**Acc SSOT:** P0 mid [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) — frozen

---

## 1. One-screen scorecard (honesty)

| Peer | n | EQ Acc | Acc Δ CI | EQ ctx | Fact ER | Claim |
|------|---|--------|----------|--------|---------|-------|
| Acc headline P0 mid | 200 | 0.706 | LR [−0.107, −0.033] | 0.396 | 0.790 | Headline SSOT — not Beat |
| Gap-close E2-B5 (**keep**) | 200 | 0.765 | tie [−0.031, +0.040] | 0.491 | 0.917 | Acc-tie labeled — **not** Parity |
| E2 medical-full | 2062 | 0.739 | LR [−0.069, −0.017] | 0.472 | 0.918 | Scale — LR Acc |
| G1 gold-compat | 200 | 0.764 | tie | **0.461** | 0.917 | REJECT (ctx tax) |
| 083 PRODUCT_QUERY_API_v1 | — | — | — | — | — | Product Equal (hl/ll + chat) |

**Parity (Equal) gates:** Acc CI ≥ tie ∧ ctx≥0.50 ∧ Fact ER≥LR−0.03 on medical-mid.  
**Beat (exceed):** same on medical-full with Acc CI EQ-ahead — only then Acc `publish/latest`.

E2 mid needs **ctx +0.009** and Fact ER ≥ LR−0.03 without Acc tax. Mid Acc tie alone ≠ Equal.

---

## 2. Forbidden

NF · dense BM25=0 · post_truncate · D1–D3 · TOPIC_* / soft Mix · B7–B9 · B10 Acc promote · F4 always-on · synonym Acc fishing · silent Acc B5 overwrite · Acc `publish/latest` without Beat gates · heuristic KEYWORD as Acc default.

---

## 3. Program

| Step | Work | Exit |
|------|------|------|
| H0 | This memo + index | Done when linked |
| H1 | Fact LR-win autopsy + Acc binary 083 verify | **Locked** [`E2_CHAT_SPLIT_v1`](../e2e/artifacts/forensics/084-fact-lr-wins.md) |
| H2 | Rebuild Acc on 083 `chat(system,user)`; COMPLETE_BLOB off | Unit + E2 pins unchanged |
| H3 | Labeled E2 medical-mid | Parity / keep / REJECT |
| H4 | medical-full only if H3 Parity | Beat or SCALE label |

**Locked confound:** Acc binary ships 083 generate role-split (LR `kg_query` system/user). Query pins = E2 occ. No packing.

---

## 4. H3 mid gate (`T041648Z`)

| Gate vs E2-B5 | Result |
|---------------|--------|
| Acc CI not LR-ahead; EQ Acc ≥ E2−0.01 | **PASS** — EQ 0.792 · CI [−0.016, +0.048] |
| ctx ≥ 0.50 (Parity) or ≥ E2−0.01 | **FAIL** — 0.473 |
| Fact ER ≥ LR−0.03 | **PASS** — 0.950 vs LR 0.963 |
| Acc `publish/latest` | Untouched (P0 mid) |

**Verdict:** REJECT Parity (ctx tax). Acc CI improved labeled. **H4 medical-full skipped.** No packing reopen.

---

## 5. Follow-on

**[085](./085-fairness-concurrency-equal-stop.md)** fairness mid (`T043401Z`) recovered ctx to **0.488** (near E2) under concurrency=4 — confirms 084 ctx tax was mostly concurrency. Parity still FAIL → **Acc Equal STOP**. Product Equal remains 083.
