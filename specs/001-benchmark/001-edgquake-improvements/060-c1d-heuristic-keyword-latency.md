# 060 — C1d heuristic KEYWORD path (product latency; not Acc promote)

**Status:** Measured ✓ · Acc Fact peer unchanged  
**Date:** 2026-07-21  
**Archive:** [`T014632Z`](../e2e/artifacts/history/smoke-20260721T014632Z/)  
**Cross-ref:** [059](./059-c1b-latency-ceiling-keyword-embed.md) · LightRAG KEYWORD role

---

## 1. First principles

| Law | Detail |
|-----|--------|
| C1b | keyword LLM p50 **1782** ms under Acc Mistral |
| LR | KEYWORD = ultra-fast non-thinking model |
| Experiment | One confound: `KEYWORD_MODE=heuristic` on C1b (BM25-all) |

---

## 2. Measured (`T014632Z`)

| Stage | C1b | C1d | Note |
|-------|----:|----:|------|
| **keyword** | 1782 | **0** | Law✓ — LLM skipped |
| embed | 2212 | 2180 | flat |
| retrieve | 539 | 983 | ↑ (weaker Mix keywords?) |
| rerank | 9 | 9 | BM25 |
| generate | 2421 | **2985** | still ceiling |
| EQ/LR p50 | 3.91× | **4.08×** | no wall win |

**Verdict:** Heuristic keyword path proves the stage. It is **not** a publishable latency win alone — generate (+ retrieve variance) dominates. Prefer a **fast KEYWORD LLM** (nano/local) that keeps Mix quality over pure heuristic for product default.

---

## 3. Implementation

| Piece | Location |
|-------|----------|
| `EDGEQUAKE_KEYWORD_MODE=llm\|heuristic` | `keywords/keyword_mode.rs` |
| Pack | `make bench001-c1d` |

Acc Fact peer keeps `KEYWORD_MODE=llm`.
