# 034 — L2 dual-list under full workspace graph (post-032/033)

**Status:** **Parity promoted** ([`T093152Z`](../e2e/artifacts/history/smoke-20260720T093152Z/)) · Beat not met  
**Cross-ref:** [026](./026-l2-sources-union-under-p2b.md) · [032](./032-workspace-graph-identity.md) · [033](./033-denser-graph-mix-packing.md) · [028](./028-first-principles-beat-roadmap.md)  
**Warm:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`

---

## 1. How close to LightRAG? (T090743Z A1+033)

| Dimension | EQ | LR | Gap |
|-----------|---:|---:|-----|
| Acc | **0.7735** | 0.7570 | Δ+0.016 · CI includes 0 → **tie** |
| evidence_recall | 0.914 | 0.965 | **−0.051** (need ≥ LR−0.03) |
| context_relevancy | 0.481 | 0.538 | need **≥0.50** |
| Complex Acc | 0.753 | 0.842 | −0.089 |
| Fact Acc | 0.782 | 0.685 | EQ ahead |

**Promote:** Beat/Parity both need ctx≥0.50 ∧ recall≥LR−0.03. **Neither met.** Acc alone is not enough (S4).

---

## 2. First-principles diagnosis

GraphRAG-Bench L2 scores the **retrieved context list** (API `sources` / prediction `context`), not the CE prompt order. Under A1/P2b:

1. CE admits top-k=30 into `context.chunks` (prompt / Acc).
2. Dual-list (026) builds `citation_chunks = CE ∪ Mix[:K]` for L2.
3. **Bug:** `build_sources_from_context` re-truncatedates to `rerank_top_k=30` whenever `reranked=true`. When CE already fills 30, **Mix fill is dead** → Fact ER flat on 026 S0.

Law violated: **L2 membership list ≠ CE prompt list** (dual-list contract). Truncating citations to CE top-k collapses the two lists.

Not FAQ. Not soft Mix Acc fishing. Not packing-knob stacking.

---

## 3. Fix (one confound)

- Skip `rerank_top_k` truncate when `citation_chunks` is set.
- Acc step **`a1l2`**: A1 (`rr_cer`) + `EDGEQUAKE_L2_SOURCES_UNION=1` on B3b+033 warm WS.
- Prompt / Acc path unchanged (still CE-ordered `context.chunks`).

Non-goals: BM25 FactReplace heuristics, protect↑ fishing, FAQ induce, force re-ingest (disk &lt;15 Gi).

---

## 4. Gates

| Outcome | Gate |
|---------|------|
| **Beat** | CI excludes 0 EQ ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| **Parity** | CI includes 0 ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| Step | Acc ≥ T090743Z−0.02 (0.753) · Fact ER ↑ vs 0.80 |

Reproduce:

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
export BENCH001_ACC_QUERY_CONCURRENCY=4
./tools/bench001/scripts/run_p_ladder_acc.sh a1l2
```

---

## 5. Results

| Run | Acc | recall | ctx | Fact ER | Note |
|-----|----:|-------:|----:|--------:|------|
| T090743Z A1+033 | **0.7735** | 0.914 | 0.481 ✗ | 0.80 | dual-list off |
| T092505Z a1l2 | 0.7239 | 0.915 | **0.506** ✓ | 0.80 | UNION works (ctx↑); Fact ER flat; Acc tax → **no promote** |
| T092930Z a1lr | **0.7583** | 0.928 | **0.506** ✓ | 0.80 | recall miss LR−0.03 by **0.005**; Acc OK |
| **T093152Z a1lrl2** | 0.7178 | **0.9325** ✓ | **0.525** ✓ | **0.85** | **Parity** (CI includes 0); Acc point tax |

**Diagnosis after a1l2:** citation truncate was real (context chars ~88k→142k Fact). Fact gold is mostly **absent from Mix first-stage**, not only CE-dropped → dual-list alone cannot lift Fact ER.

**Diagnosis after a1lr:** LR VECTOR budget lifts overall recall (0.914→0.928) and clears ctx; Fact ER still 0.80.

**Parity package (`a1lrl2`):** A1 + LR VECTOR budget + Mix∪CE dual-list (citation truncate fix). Clears L2 gates; Acc remains a statistical tie (point estimate behind LR).

Reproduce Parity:

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
export BENCH001_ACC_QUERY_CONCURRENCY=4
./tools/bench001/scripts/run_p_ladder_acc.sh a1lrl2
```

Acc-preferred Fact peer (no dual-list tax): [035 `a1fp`](./035-fact-ce-bm25-protect.md) [`T095809Z`](../e2e/artifacts/history/smoke-20260720T095809Z/) Acc **0.775** · Fact ER **0.85**.
