# 085 — Fairness Concurrency then Acc Equal STOP (First Principles)

**Status:** F3 **REJECT Parity** · Acc Equal mid path **STOP** · Acc Beat **STOP** · Acc latest frozen  
**F3 archive:** [`T043401Z`](../e2e/artifacts/history/medical-mid-20260723T043401Z/) · peer `E2_CHAT_SPLIT_fair_v1`  
**Date:** 2026-07-23  
**Parent:** [084](./084-honest-equal-lightrag-rebench.md) · [081](./081-beat-parity-first-principles.md) · [055](./055-post-acc-ceiling-first-principles.md)  
**Warm WS:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5)  
**Keep query base:** E2 occ [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**084 labeled:** `E2_CHAT_SPLIT_v1` [`T041648Z`](../e2e/artifacts/history/medical-mid-20260723T041648Z/)  
**Acc SSOT:** P0 mid [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) — frozen  
**Product Equal:** [083](./083-lightrag-query-api-law.md) (unchanged)

---

## 1. First-principles verdict on “rebench?”

| Law | Implication |
|-----|-------------|
| `ctx_rel` is L2 retrieval-only (question vs contexts; GraphRAG-Bench) | Chat-split generate **cannot** raise Parity ctx |
| 084 FAIL was ctx 0.473 (Acc CI / Fact ER already OK) | Do **not** Acc-fish generation knobs for Equal |
| EQ contexts differ **172/200** E2→084; concurrency **4→8** | 084 ctx tax is fairness / nondeterminism leak |
| Packing levers that move ctx | Forbidden (NF / dense / post_truncate / D1–D3 / TOPIC / soft Mix) |

**Action taken:** one labeled fairness rebench — same chat-split + **E2 concurrency (4)** + eval **24**. Parity still FAIL → **Acc Equal mid path STOP**. Product Equal stays 083.

---

## 2. Forbidden (carry forward)

NF · dense BM25=0 · post_truncate · D1–D3 · TOPIC_* / soft Mix · B7–B9 · Acc latest without Beat · `response_type` Acc fishing · blind chat-split rerun without concurrency lock · packing reopen · further Acc mid for ctx.

---

## 3. Program

| Step | Work | Exit |
|------|------|------|
| F0 | This memo + index | Done |
| F1 | Ctx drift forensics | [`085-ctx-drift-chat-split.md`](../e2e/artifacts/forensics/085-ctx-drift-chat-split.md) |
| F2 | Ladder `e2-chat-split-fair` · concurrency=4 · eval=24 | Peer `E2_CHAT_SPLIT_fair_v1` |
| F3 | Labeled medical-mid `SKIP_PUBLISH_LATEST` | **REJECT Parity** |
| F4 | Honesty closeout | **Acc Equal STOP** · Acc latest frozen |

---

## 4. F3 mid gate (`T043401Z`)

| Gate vs E2-B5 | Result |
|---------------|--------|
| Acc CI not LR-ahead; EQ Acc ≥ E2−0.01 | **PASS** — EQ 0.791 · CI [−0.022, +0.050] |
| ctx ≥ 0.50 (Parity) | **FAIL** — 0.488 |
| ctx ≥ E2 − 0.01 (fairness keep) | **PASS** — 0.488 ≥ 0.481 |
| Fact ER ≥ LR − 0.03 | **FAIL** — 0.910 vs LR 0.950 (need ≥0.920) |
| Acc `publish/latest` | Untouched (P0 mid) |

**Attribution:** concurrency lock recovered ctx 0.473→0.488 (near E2). EQ contexts still drift vs E2 (~174/200) — Mix nondeterminism remains. No Acc-safe packing-free lever to ctx≥0.50.

**Verdict:** REJECT Parity. **Acc Equal mid path STOP.** No medical-full. No packing.

---

## 5. Publishable claims

- Product Equal = **083**.
- Acc CI labeled keep = E2-B5 / fair chat-split (tie, EQ point-ahead) — **not** Parity.
- Mid Parity / Beat = **unmet**; do not schedule another Acc mid for ctx.
