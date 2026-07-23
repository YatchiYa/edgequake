# 082 — Gold / Citation Compat (then Honesty Freeze)

**Status:** G1 **REJECT** (ctx tax) · mid Parity **unfinished** · Acc Beat fishing **STOP** · Acc `publish/latest` frozen  
**Date:** 2026-07-23  
**Parent:** [081](./081-beat-parity-first-principles.md) (F3/F4 REJECT · packing STOP)  
**G1 archive:** [`T024205Z`](../e2e/artifacts/history/medical-mid-20260723T024205Z/) · smoke [`T023833Z`](../e2e/artifacts/history/smoke-20260723T023833Z/)  
**Keep query base:** E2 occ on B5 [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**Acc SSOT:** P0 mid [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) · warm `8e990410-…`

---

## 1. Honesty freeze (binding claims)

**Mid Parity is unfinished.** Publishable truth stays split-peer:

| Peer | Role |
|------|------|
| Acc headline P0 mid | `publish/latest` SSOT — not Beat |
| Gap-close E2-B5 | Best Acc CI (tie) — ctx 0.491 · Fact ER 0.917 — **not** Parity |
| B6 Fact ER / B10 / F4 | Labeled only |

Do **not** claim EQ beats LightRAG until Phase G: mid Acc CI ≥ tie ∧ ctx≥0.50 ∧ Fact ER≥LR−0.03 **and** medical-full does not reopen a large LR Acc CI.

Acc Beat fishing STOP after G1 if Parity still unmet — no packing reopen.

---

## 2. Why this lever (not packing)

F1 Fact LR-wins = **100% generation** (gold already in EQ Acc context). F4 always-on REJECT. Residual structural conflict:

- Acc pins `answer_style=gold` → forbids `[N]` / chunk ids ([`judge_tune.py`](../../../tools/bench001/bench001/judge_tune.py)).
- EQ default prompt still injects citation-mandate [`grounding_instructions()`](../../../edgequake/crates/edgequake-query/src/grounding.rs).

**G1 law:** when gold extension is present, omit citation mandates (keep entailment + grounded arithmetic), strip trailing citation artifacts, gold-shape extractive fallback. Product path without gold: unchanged.

### Forbidden

NF · dense BM25=0 · post_truncate · D1–D3 · TOPIC_*/soft Mix · B7–B9 · silent B5 overwrite · cap-relation-chunks · F4 always-on · B10 Acc promote · `ANSWER_PROMPT=lightrag` as first confound.

---

## 3. Gates (G1)

```bash
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
export BENCH001_SKIP_PUBLISH_LATEST=1
export BENCH001_PUBLISH_PEER=LR_OCC_FACT_L2_G1_v1
export BENCH001_LADDER_STAGE=smoke   # then medical-mid
./tools/bench001/scripts/run_p_ladder_acc.sh lr-occ-fact-l2
```

| Gate | Target vs E2-B5 | G1 result |
|------|-----------------|-----------|
| Acc CI | not clearly LR-ahead; EQ Acc ≥ E2 − 0.01 | PASS · EQ 0.764 · CI [−0.057, +0.010] |
| ctx / Fact ER | ≥ E2 − 0.01 (no L2 tax) | **FAIL ctx 0.461** (E2 0.491); Fact ER 0.917 PASS |
| Parity candidate | Acc CI ≥ tie ∧ ctx≥0.50 ∧ Fact ER≥LR−0.03 | **No** |
| Verdict | — | **REJECT** → H1 honesty freeze; gold-compat code kept |

---

## 4. Code map

| Concern | Files |
|---------|-------|
| Gold detect / grounding variant / strip | `edgequake-query/src/grounding.rs` |
| Prompt + fallback + post-strip | `edgequake-query/src/engine_impl/prompt.rs` |
| Acc gold text | `tools/bench001/bench001/judge_tune.py` |
