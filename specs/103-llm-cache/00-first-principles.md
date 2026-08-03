# SPEC-103 — First Principles

> **Cross-refs**: [WHY](00-why.md) · [Roadmap](03-implementation-roadmap.md) · [063](../001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md) · [064](../001-benchmark/001-edgquake-improvements/064-product-ttft-cache-batch-embed.md) · [llm-cache-scope](../../docs/data-layer/llm-cache-scope.md)

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-C1** | Cache is a **content-addressed recomputation guard**, never document SSOT. |
| **LAW-C2** | Storage key = `{mode}:{cache_type}:{hash}-cache` with `cache_type ∈ {keywords,query}` (LR shape + SPEC-091 `-cache` suffix). |
| **LAW-C3** | **`query`** hash = full built RAG prompt (context-inclusive — safer than LR). **`keywords`** hash = query + mode + model (+ language). Use delimited arg join (`\x1e`) to avoid LR-style adjacent-field collisions. |
| **LAW-C4** | Persistence SSOT = `public.llm_cache` (namespace-scoped; no workspace column). |
| **LAW-C5** | L1 memory + L2 postgres; cache errors never fail the query (warn + recompute). |
| **LAW-C6** | Master `EDGEQUAKE_LLM_CACHE` default **on**; overrides `EDGEQUAKE_KEYWORD_CACHE`, `EDGEQUAKE_QUERY_ANSWER_CACHE`. |
| **LAW-C7** | Acc / `c1cold`: pin `EDGEQUAKE_LLM_CACHE=0` — warm cache is never a latency Beat claim. |
| **LAW-C8** | Observability: `keyword_cache_hit` + `answer_cache_hit` on query stats. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | `LlmResponseCache` owns get/set; extractors/pipeline own when to call; storage owns SQL. |
| **O** | New `cache_type` = extend enum + hash helper; callers unchanged. |
| **L** | Memory-only and tiered backends satisfy the same trait. |
| **I** | No provider-wide “complete if cache” facade required; query-path ports only. |
| **D** | Query depends on a store port, not `sqlx` / pool types. |
| **DRY** | One key helper, one envelope (`return` / `cache_type`), one flag resolver, one L2 table. |

## Env resolution (normative)

```text
fn resolve_llm_cache_flags() -> (master, keywords, answer):
  master = env_truthy(EDGEQUAKE_LLM_CACHE, default=true)
  keywords = master && !env_falsey(EDGEQUAKE_KEYWORD_CACHE)
  answer = master && env_answer_enabled()
    # answer: if EDGEQUAKE_QUERY_ANSWER_CACHE unset → follow master
    #         if set falsey → off; if set truthy → on
```

## Non-goals

- Semantic similarity answer cache  
- Entity-extract / summary unification (v1)  
- Warm Acc ≤1.5× claims vs LightRAG  
