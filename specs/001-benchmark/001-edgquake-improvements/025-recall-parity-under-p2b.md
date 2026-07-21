# 025 — Evidence-Recall Parity under P2b

**Status:** Ladder complete — **no Acc promote** (recall still short; combo Acc-toxic)  
**Date:** 2026-07-20  
**Trigger:** [024 Q4](./024-acc-parity-beat-plan.md) `T024233Z` — Acc 0.756 · ctx_rel 0.506 · recall **0.914** vs LR **0.987** (need ≥0.957)  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

---

## 1. Diagnosis

CE admission on P2b demotes Mix Fact/Summarize chunks (Fact recall ~0.85 vs BM25 ~0.95). Acc/ctx_rel stay OK; L2 `sources` membership swaps → evidence_recall miss.

```text
Mix RRF → CE + min_rerank_score=0.1 → PROTECT_FIRST=12 → truncate
                │
                └─ gold Fact/Summarize tails drop from L2 sources
```

---

## 2. Gates (same as 024)

| Outcome | Gate |
|---------|------|
| **Beat** | CI excludes 0 EQ **and** ctx_rel ≥ 0.50 **and** recall ≥ LR − 0.03 |
| **Parity** | CI includes 0 **and** ctx_rel ≥ 0.50 **and** recall ≥ LR − 0.03 |
| Per-step | Acc ≥ Q4 − 0.02 (0.736); prefer Fact recall ≥ 0.92 |

---

## 3. Ladder (one confound each on P2b)

```bash
make bench001-r0   # PROTECT_FIRST=20
make bench001-r1   # MIN_RERANK_SCORE=0
make bench001-r2   # MIN_CHUNK_BUDGET_RATIO=0.55
make bench001-r3   # CI decision on best R*
```

| Step | Knob | Success |
|------|------|---------|
| **R0** | `EDGEQUAKE_RERANK_PROTECT_FIRST=20` | recall ≥ LR−0.03; ctx≥0.50; Acc≥0.736 |
| **R1** | `EDGEQUAKE_MIN_RERANK_SCORE=0` | same (if R0 miss) |
| **R2** | `EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO=0.55` | same (if R1 miss) |
| **R3** | Best single winner (or P2b if all miss) | Beat/Parity → promote |

**Do not stack** until one confound clears. Forbidden: Fact VECTOR, P4 stack, BM25+path, RR default.

---

## 4. Ledger

| Step | Archive | EQ Acc | ctx | recall EQ/LR | Notes |
|------|---------|--------|-----|--------------|-------|
| **R0** | `T025415Z` | 0.732 | 0.519 | 0.911 / 0.964 | protect20 alone — Fact recall still 0.80 |
| **R1** | `T025649Z` | 0.741 | 0.494 | **0.928** / 0.966 | Best single recall; ctx just under 0.50 |
| **R2** | `T025940Z` | 0.714 | 0.506 | 0.924 / 0.986 | chunk floor — Acc tax |
| **R3** | `T030223Z` | 0.693 | 0.506 | 0.928 / 0.961 | protect20+min_rerank0 — Acc toxic; recall still −0.003 vs gate |

**Decision:** No promote. Acc headline stays P0 BM25 / path off. Best peer remains **P2b Q4** `T024233Z` (Acc 0.756, ctx 0.506). CE admission knobs move recall toward LR but do not clear ≥LR−0.03 without Acc collapse.

**Carry-forward:** Investigate L2 source emission / CE set membership (Fact Jaccard), not further protect/min_score fishing.

---

## 5. Non-goals

- Soft Mix Acc fishing · Fact VECTOR restack · latency SLO this sprint  
