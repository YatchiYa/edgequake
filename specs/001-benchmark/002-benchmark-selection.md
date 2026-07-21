# 002 — Benchmark Selection (July 2026)

**Cross-ref:** [001 First Principles](./001-first-principles.md) · [004 Dataset](./004-dataset-and-fixtures.md)

---

## 1. Decision (locked)

**Primary / default:** GraphRAG-Bench (ICLR 2026).

Sources:
- Dataset: https://huggingface.co/datasets/GraphRAG-Bench/GraphRAG-Bench
- Eval: https://github.com/GraphRAG-Bench/GraphRAG-Benchmark
- Paper: https://arxiv.org/abs/2506.05690 (*When to use Graphs in RAG*)

---

## 2. Selection criteria (first principles)

| Criterion | Weight | GraphRAG-Bench | UltraDomain + LR batch_eval | HybridRAG-Bench | MMLongBench (047) |
|-----------|--------|----------------|-----------------------------|-----------------|-------------------|
| Official LightRAG baseline published | High | Yes | Yes (paper) | Yes (wrapper) | No |
| Gold answers (reliable Acc) | High | Yes | No (pairwise judge) | Yes | Yes (EQ-only) |
| Easy to launch | High | HF JSON + eval scripts | Large ingest + judge | Neo4j + vLLM + siblings | Heavy PDFs |
| Quick smoke path | High | `--sample` / 40 IDs | Mix still ~619k tokens | Heavy index | chart-8 OK but EQ-only |
| Hybrid / graph necessity | High | Four difficulty levels | High-level sensemaking | Hybrid KG+text | Multimodal PDF |
| Dual-SUT fairness | High | Same text corpus | Same | Text-only OK; KG path biased | N/A |

---

## 3. Rejected as SPEC-001 default

### UltraDomain (TommyChien/UltraDomain) + LightRAG Reproduce.md

- **What it is:** Official LightRAG EMNLP evaluation — pairwise LLM win-rates on Comprehensiveness / Diversity / Empowerment / Overall.
- **Why not default:** No ground truth; judge variance; Legal domain ~5M tokens; fails “reliable + quick.”
- **When to use:** Separate paper-parity annex outside SPEC-001 headline claims.

### HybridRAG-Bench (junhongmit/HybridRAG-Bench, arXiv:2602.10210)

- **What it is:** 2026 hybrid knowledge (text + KG) multi-hop framework with LightRAG wrappers.
- **Why not default:** Neo4j + vLLM + sibling HippoRAG/LightRAG/GraphRAG clones — fails “easy to launch.”
- **When to use:** Research track after SPEC-001 smoke is green.

### MMLongBench-Doc (SPEC-047)

- **What it is:** Long multimodal PDF Acc/F1 for EdgeQuake RAG adaptation.
- **Why not here:** EQ-only; not a LightRAG head-to-head; different task identity.

---

## 4. What GraphRAG-Bench gives us

| Asset | Detail |
|-------|--------|
| Corpora | `medical` (1 context blob), `novel` (20 contexts) |
| Questions | ~2062 medical + ~2010 novel; types Fact / Reasoning / Summarize / Creative |
| Evidence | Gold answer + evidence spans per question |
| Metrics | Acc, ROUGE-L, Coverage, Faithfulness (by type) |
| LightRAG example | `Examples/run_lightrag.py` with `--sample N` |

---

## 5. Honesty banner (mandatory in scorecards)

> This scorecard compares EdgeQuake `mix` vs LightRAG `mix` on GraphRAG-Bench under pinned models. It is **not** an UltraDomain win-rate table and **not** an MMLongBench-Doc LVLM score.
