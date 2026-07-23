# Ablation — E2_CHAT_SPLIT_fair_v1 (085)

**Step:** e2-chat-split-fair  
**Stage:** medical-mid  
**Archive:** `medical-mid-20260723T043401Z`  
**Pins:** 084 chat-split + `eq_query_concurrency=4` + `eval_concurrency=24` (match E2)  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Acc `publish/latest`:** skipped (P0 mid SSOT unchanged)

## Results vs E2-B5 keep (`T133053Z`) and 084 (`T041648Z`)

| Metric | E2-B5 | 084 chat-split (conc=8) | **085 fair (conc=4)** | Gate |
|--------|-------|-------------------------|------------------------|------|
| EQ Acc | 0.765 | 0.792 | **0.791** | ≥ E2 − 0.01 → **PASS** |
| Acc Δ CI | [−0.031, +0.040] | [−0.016, +0.048] | **[−0.022, +0.050]** | not LR-ahead → **PASS** |
| EQ ctx_rel | 0.491 | 0.473 | **0.488** | ≥0.50 Parity **FAIL** · ≥E2−0.01 keep **PASS** |
| Fact ER | 0.917 / LR 0.953 | 0.950 / LR 0.963 | **0.910 / LR 0.950** | ≥ LR − 0.03 → **FAIL** (0.910 < 0.920) |

## Verdict

- [x] **REJECT Parity** (ctx 0.488 < 0.50; Fact ER miss)
- Fairness concurrency recovered ctx from 0.473 → 0.488 (near E2) — confirms 084 ctx tax was mostly concurrency leak
- Acc CI remains labeled keep candidate — **not** Equal / Beat
- Do **not** Acc promote · do **not** open packing · **no medical-full**
- **Acc Equal mid path STOP** (first principles: no Acc-safe packing-free path to ctx≥0.50)

**Claim:** product Equal remains **083**. Mid Acc Parity unfinished. Not Beat.
