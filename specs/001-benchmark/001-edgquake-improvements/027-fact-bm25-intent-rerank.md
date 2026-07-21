# 027 — Fact L2 BM25 under P2b (prompt stays CE)

**Status:** Closed — **no promote** (best L2 = T0d strict `T035516Z`; Acc peer still Q4 P2b)  
**Date:** 2026-07-20  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

---

## 1. Diagnosis

CE under P2b holds Acc/ctx but Fact ER ≈0.85. BM25 Acc Fact ER ≈0.95. Dual-list must keep **CE prompt** (Acc) and improve **L2 sources** without Mix-noise flooding.

---

## 2. Gates (unchanged)

| Outcome | Gate |
|---------|------|
| **Beat** | CI excludes 0 **EQ** **and** ctx≥0.50 **and** recall≥LR−0.03 |
| **Parity** | CI includes 0 **and** ctx≥0.50 **and** recall≥LR−0.03 |

---

## 3. Ledger

| Step | Archive | Acc | Fact ER | recall | ctx | Notes |
|------|---------|-----|---------|--------|-----|-------|
| T0 prompt BM25 | `T032829Z` | 0.720 | 0.90 | 0.934 | 0.494 | Acc tax |
| T0b CE-first ∪ | `T033613Z` | 0.724 | 0.85 | 0.926 | **0.50** | Fact flat |
| T0c replace all | `T034245Z` | 0.727 | **0.90** | 0.904 | 0.40 | ctx crash |
| T0d heuristic OR | `T034914Z` | 0.701 | **1.0** | **0.958** | 0.44 | over-route |
| **T0d LLM-only** | **`T035516Z`** | 0.722 | **0.95** | 0.939 | **0.512** | recall −0.002 vs gate; CI favors LR |
| Q4 P2b peer | `T024233Z` | **0.756** | 0.85 | 0.914 | 0.506 | Acc peer |

---

## 4. Verdict

- **Do not promote** FactReplace / L2 BM25 as Acc headline.
- **Labeled peer pack for L2:** P2b + `EDGEQUAKE_L2_BM25_UNION=1` + `EDGEQUAKE_L2_BM25_MODE=fact_replace` (LLM factual only).
- **Acc peer unchanged:** P2b alone (`T024233Z`).
- Blocker to Parity/Beat: overall Acc Δ still LR-favored; only 5/10 GraphRAG Fact rows get `query_intent=factual`.

---

## 5. Env

| Env | Meaning |
|-----|---------|
| `EDGEQUAKE_L2_BM25_UNION=1` | enable dual-list |
| `EDGEQUAKE_L2_BM25_MODE` | `union` \| `replace` \| `fact_replace` |
| `EDGEQUAKE_FACT_RERANKER=bm25` | T0 prompt path (closed — Acc toxic) |

```bash
make bench001-t0d   # fact_replace (LLM factual → BM25 L2)
make bench001-t1    # Acc CI (default fact_replace)
```
