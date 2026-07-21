# 062 — C1e fast KEYWORD LLM (LightRAG role law)

**Status:** Measured ✓ · **wall latency REJECT** · Acc Fact peer unchanged  
**Date:** 2026-07-21  
**Archive:** [`T020802Z`](../e2e/artifacts/history/smoke-20260721T020802Z/)  
**Cross-ref:** [061](./061-lightrag-law-first-principles-eq.md) Idea A · [059](./059-c1b-latency-ceiling-keyword-embed.md) · [060](./060-c1d-heuristic-keyword-latency.md)

---

## 1. First principles

| Law | Detail |
|-----|--------|
| LR | KEYWORD = ultra-fast non-thinking model ≠ QUERY |
| EQ gap | Workspace `llm_roles.keyword` existed; Acc/default paid Query model (~1.8s) |
| Experiment | One confound on C1b: `KEYWORD_LLM=ministral-3b-latest` (same Mistral provider) |
| Success | keyword p50 ≪ 400 ms **and** wall ↓ vs C1b |
| Acc | Not promote — Acc Fact peer stays B5+`a1fp` |

---

## 2. Measured (`T020802Z`)

| Stage | C1b | C1e | Note |
|-------|----:|----:|------|
| **keyword** | 1782 | **2035** | Law✓ model used; **slower** under Acc concurrency |
| embed | 2212 | 2313 | flat |
| retrieve | 539 | 529 | flat |
| rerank | 9 | 9 | BM25 |
| generate | 2421 | **3039** | still ceiling |
| EQ/LR p50 | 3.91× | **4.75×** | no wall win |
| EQ Acc | 0.712 | 0.742 | tax labeled; not Acc peer |

**Pins confirmed:** `keyword_llm_provider=mistral`, `keyword_llm_model=ministral-3b-latest`.  
**Logs confirmed:** `Creating … model=ministral-3b-latest` on `/api/v1/query`.

**Verdict:** Process-env KEYWORD role is **product-ready (Law✓)**. Remote `ministral-3b-latest` is **not** an ultra-fast KEYWORD pin for EQ’s long keyword+intent prompt under Acc concurrency — often more verbose keywords → more tokens → worse stage p50. Prefer **local / true-nano / short-output KEYWORD** next, or move to Idea B (batch embed) / Idea C (TTFT).

---

## 3. Implementation

| Piece                                  | Location                                                                |
| ----------------------------------------| -------------------------------------------------------------------------|
| `env_keyword_role_llm()`               | `edgequake-core/src/llm_roles.rs`                                       |
| Resolve env → workspace → Query        | `query_execution.rs` (`resolve_keyword_llm`)                            |
| Process-lifetime cache                 | `env_keyword_llm_cached` (`OnceLock`)                                   |
| Nested JSON unwrap (`data`/`response`) | `keywords/llm_extractor.rs`                                             |
| Acc export + override                  | `start_acc_backend.py`                                                  |
| Pack                                   | `make bench001-c1e` → `EDGEQUAKE_KEYWORD_LLM_MODEL=ministral-3b-latest` |
| Pins                                   | `fair_pins.keyword_llm_{provider,model}`                                |

```bash
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
make bench001-c1e
```

---

## 4. Next (still Horizon C)

1. **Local / OpenAI nano KEYWORD** when a real fast endpoint+key is available (not Acc Mistral dual-tax)  
2. **[061 Idea B](./061-lightrag-law-first-principles-eq.md)** — single-batch embed + batch cache (`c1f`)  
3. **[061 Idea C](./061-lightrag-law-first-principles-eq.md)** — TTFT metric (`c1g`) — generate still owns the ceiling  

Acc Fact peer unchanged.
