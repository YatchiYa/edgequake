# 084 Fact LR-win autopsy (E2-B5) + Acc binary verify

**Date:** 2026-07-23  
**Source slice:** [`f1-e2-mid`](./f1-e2-mid/) · archive `medical-mid-20260722T133053Z`  
**Warm WS:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Honesty:** labeled forensics — not Acc Beat

---

## 1. F1 reuse (E2 mid)

| Metric | Value |
|--------|-------|
| Fact LR-wins n | 10 |
| membership_share | **0.0** |
| generation_share | **1.0** |
| EQ empty answers | 0 |
| EQ ctx | 0.491 |
| EQ Fact ER | 0.917 vs LR 0.953 |
| Gold coverage on Fact LR-wins | often **1.0** (gold tokens already in EQ Acc context) |

**Law:** residual Acc gap is **generation / role formatting**, not Mix packing admission.

---

## 2. Acc binary vs 083 chat-split

| Check | Result |
|-------|--------|
| Running Acc `/health` build_timestamp | `2026-07-23T02:34:03Z` |
| `edgequake/target/release/edgequake` mtime | `2026-07-23T10:37:21Z` |
| `prompt.rs` (083 chat split) mtime | `2026-07-23T11:46:25Z` (**newer than binary**) |
| `EDGEQUAKE_ANSWER_COMPLETE_BLOB` in Acc proc | unset |
| `EDGEQUAKE_KEYWORD_MODE` | `llm` (correct Acc) |
| Probe `hl_keywords` override `keyword_time_ms` | **752** (skip not live on running Acc) |

**Conclusion:** Acc path is still on **pre-083 generate** (`complete` blob). Source tree has LR-shaped `chat(system, user)` default; Acc has not been rebuilt onto it.

---

## 3. Locked single confound

**Peer id:** `E2_CHAT_SPLIT_v1`  
**Confound:** Rebuild Acc backend from current tree so generate uses 083 system/user `chat()` (COMPLETE_BLOB remains off). Query pins = E2 occ (`lr-occ-fact-l2`) unchanged. No packing / TOPIC / heuristic KEYWORD.

**Not chosen:** `response_type` Acc pin alone — moot until chat-split binary is live.

---

## 4. Gate after H3

| Gate vs E2-B5 | Pass |
|---------------|------|
| Acc CI | not clearly LR-ahead; EQ Acc ≥ E2 − 0.01 |
| ctx_rel | ≥ 0.50 (Parity) or ≥ E2 − 0.01 keep |
| Fact ER | ≥ LR − 0.03 |
| Acc `publish/latest` | **SKIP** |
