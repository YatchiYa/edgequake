# 035 — Fact CE∩BM25 protect (no dual-list Acc tax)

**Status:** **Acc-preferred Fact package** [`T095809Z`](../e2e/artifacts/history/smoke-20260720T095809Z/) · Parity L2 still needs dual-list or further recall work  
**Cross-ref:** [034](./034-l2-dual-list-under-full-ws-graph.md) · [027](./027-fact-bm25-intent-rerank.md) · [025](./025-recall-parity-under-p2b.md) · LightRAG operate.py (naive-favor when rerank off)  
**Warm:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4` · query-only (disk &lt;15 Gi)

---

## 1. Observation

| Run | Acc | Fact ER | recall | ctx | Dual-list |
|-----|----:|--------:|-------:|----:|-----------|
| T090743Z A1 | **0.773** | 0.80 | 0.914 | 0.481 | off |
| T092930Z a1lr | 0.758 | 0.80 | 0.928 | **0.506** | off |
| T093152Z a1lrl2 | 0.718 | **0.85** | **0.933** | **0.525** | on · Acc tax |

Parity exists only with dual-list Acc tax. Goal: **Fact ER↑ + Acc point↑** with union **off**.

---

## 2. First-principles diagnosis

1. GraphRAG Fact rows need **exact lexical** evidence (drug names, regimen acronyms, “hematopathologist”).
2. CE underweights exact matches vs BM25 (Rau et al.; 027 ledger Fact ER CE≈0.85 vs BM25≈0.95).
3. LightRAG Acc peer: **rerank off** → Mix chunk order favors naive/lexical.
4. EQ Acc peer: CE on → Fact gold demoted out of top-k **sources** (= prompt set when dual-list off).
5. Full Fact→BM25 (027 T0) Acc-toxic. Dual-list (034) Acc point tax. Forbidden here.

**Law:** For Factual intent, first-stage used for `protect_first` must be **BM25-ordered Mix**, while final LLM order stays **CE**. Membership (Acc context + L2 sources) gains Fact gold; CE still ranks for generation.

---

## 3. Fix

| Env | Behavior |
|-----|----------|
| `EDGEQUAKE_FACT_PROTECT_BM25=1` | If `query_intent=Factual`, BM25-reorder Mix chunks **before** CE+protect |

Non-goals: `L2_SOURCES_UNION`, Fact→BM25 prompt replace, FAQ, force-ingest, protect↑ fishing.

Ladder: `a1fp` = A1 + `FACT_PROTECT_BM25=1` (no LR budget, no union).

---

## 4. Gates

| Outcome | Gate |
|---------|------|
| **Beat** | CI excludes 0 EQ ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| **Parity** | CI includes 0 ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| Step | Acc ≥ T090743Z−0.02 (0.753) · Fact ER ≥ 0.85 · no dual-list |

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
export BENCH001_ACC_QUERY_CONCURRENCY=4
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp
```

---

## 5. Results

| Run | Acc | recall | ctx | Fact ER | Note |
|-----|----:|-------:|----:|--------:|------|
| **T095809Z a1fp** | **0.7753** | 0.926 | **0.500** ✓ | **0.85** | **Acc peer** · Fact ER = dual-list · no Acc tax |
| T100053Z a1fplr | 0.7384 | 0.918 | **0.519** ✓ | 0.85 | stack Acc-toxic · **reject** |

**Decision:** Promote **`a1fp`** as Acc-preferred Fact package (Fact ER 0.85, Acc 0.775, ctx≥0.50, dual-list off). Overall recall still short of LR−0.03 → not L2 Parity alone. L2 Parity remains [034 `a1lrl2`](./034-l2-dual-list-under-full-ws-graph.md) (Acc point tax). Do not stack LR budget onto Fact protect. Query-only recall close [036](./036-a1fp-recall-without-dual-list.md) exhausted — next ingest when disk OK.
