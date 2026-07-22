# 026 — L2 Sources Union under P2b (CE membership)

**Status:** Closed — **no promote** (S0 `T031658Z`) → next [027](./027-fact-bm25-intent-rerank.md)  
**Date:** 2026-07-20  
**Trigger:** [025](./025-recall-parity-under-p2b.md) — CE protect/min_rerank cannot clear recall≥LR−0.03 without Acc tax  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

---

## 1. Diagnosis

L2 `evidence_recall` scores the **same post-CE truncated chunk set** used for API `sources`. CE admission swaps Fact gold out of that set. Protect widens the **prompt** → Acc tax. Fix: **dual-list** — prompt stays CE-ordered; L2 sources = Mix∪CE.

```text
Mix RRF ──clone──► citation_chunks = Mix[:K] ∪ CE_final  → L2 sources
   │
   └─ CE + protect + truncate → context.chunks → LLM prompt / Acc
```

---

## 2. Gates

| Outcome | Gate |
|---------|------|
| **Beat** | CI excludes 0 EQ **and** ctx≥0.50 **and** recall≥LR−0.03 |
| **Parity** | CI includes 0 **and** ctx≥0.50 **and** recall≥LR−0.03 |
| Step | Acc ≥ Q4−0.02 (0.736) |

---

## 3. Ladder

```bash
make bench001-s0   # P2b + L2_SOURCES_UNION=1
make bench001-s1   # Acc CI decision (promote if Beat/Parity)
```

| Env | Default |
|-----|---------|
| `EDGEQUAKE_L2_SOURCES_UNION` | `0` (off) |
| `EDGEQUAKE_L2_SOURCES_MIX_TOP_K` | `30` |

---

## 4. Ledger

| Step | Archive | EQ Acc | recall | ctx | Notes |
|------|---------|--------|--------|-----|-------|
| S0 | `smoke-20260720T031658Z` | 0.726 | 0.929 | 0.488 | Fact ER **0.85 flat**; ctx tax; no promote |
| S1 | skipped | — | — | — | redundant after S0 miss → 027 |

---

## 5. Non-goals

- More protect fishing · Fact VECTOR · P4 stack · BM25+path  
