# 085 — Ctx drift forensics (E2-B5 vs E2_CHAT_SPLIT)

**Date:** 2026-07-23  
**Archives:** E2 keep [`T133053Z`](../history/medical-mid-20260722T133053Z/) · 084 chat-split [`T041648Z`](../history/medical-mid-20260723T041648Z/)  
**Warm WS:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Honesty:** labeled forensics — not Acc Beat

---

## 1. Pin diff (scorecard)

| Pin | E2 `T133053Z` | 084 `T041648Z` |
|-----|---------------|----------------|
| `profile_id` | `LR_OCC_FACT_L2_v1_…` | `E2_CHAT_SPLIT_v1_…` |
| `eq_query_concurrency` / `query_concurrency` | **4** | **8** |
| `eval_concurrency` | **24** | **16** |
| `kg_chunk_pick_timing` | absent | `per_arm` (serialize default) |
| `mix_intent_weights` | absent | `False` |
| Workspace | same B5 | same B5 |

Ladder Acc default `--query-concurrency "${BENCH001_ACC_QUERY_CONCURRENCY:-8}"` explains the 8. E2 keep ran at 4.

---

## 2. Context hash mismatch (`predictions_*.json`)

| SUT | Common n | Context SHA mismatch |
| -----| ----------| ----------------------|
| EQ  | 200      | **172**              |
| LR  | 200      | **2**                |

Chat-split is a **generate** confound. `ctx_rel` scores question vs retrieved contexts only (GraphRAG-Bench retrieval eval). A pure generate change cannot rewrite EQ Mix contexts on 172/200 rows.

**Conclusion:** 084 EQ ctx 0.491→0.473 is **fairness / retrieval nondeterminism** (concurrency + judge noise), not a chat-split generation tax. LR ctx also drifted slightly (judge noise; contexts almost identical).

---

## 3. By-type EQ `context_relevancy`

| Type | E2 | 084 |
|------|----|-----|
| Fact | 0.550 | 0.545 |
| Complex | 0.535 | 0.515 |
| Summarize | 0.550 | 0.515 |
| Creative | 0.330 | 0.315 |

Diffuse small drops — consistent with retrieval reorder / judge variance under higher query concurrency, not a single Mix packing knob.

---

## 4. Locked fairness confound (F2/F3)

**Peer:** `E2_CHAT_SPLIT_fair_v1`  
**Confound:** same 084 chat-split + E2 occ pins + **`BENCH001_ACC_QUERY_CONCURRENCY=4`** + **`BENCH001_QUERY_CONCURRENCY=4`** + **`BENCH001_EVAL_CONCURRENCY=24`** / `ACC_EVAL=24`.  
**Not chosen:** packing reopen · `response_type` Acc fishing · Acc latest promote.

### F3 outcome (`T043401Z`)

| Metric | 084 (conc=8) | 085 fair (conc=4) |
|--------|--------------|-------------------|
| EQ ctx_rel | 0.473 | **0.488** (near E2 0.491) |
| EQ Acc | 0.792 | 0.791 |
| Fact ER | 0.950 | 0.910 |
| Parity | REJECT | **REJECT** (ctx&lt;0.50 + Fact ER miss) |

EQ context hashes still drift vs E2 (~174/200) under concurrency=4 — Mix nondeterminism remains. **Acc Equal STOP.**

