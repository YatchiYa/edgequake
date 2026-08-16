# SPEC-103 — LightRAG-parity LLM Cache

**Status:** IMPLEMENTED · v1 query-path (`keywords` + `query`) · `make spec103-llm-cache-proof`  

**Product pin:** persistent L1+L2 LLM response cache (LR `enable_llm_cache` law)  
**Inherits:** [SPEC-091 llm_cache](../091-simplify-data-layer/) · [docs/data-layer/llm-cache-scope.md](../../docs/data-layer/llm-cache-scope.md) · [063 Acc fairness](../001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md) · [064 product polish](../001-benchmark/001-edgquake-improvements/064-product-ttft-cache-batch-embed.md) · [SPEC-026 MM cache](../026-egdequake-vs-lightrag/)

---

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Unify keyword + answer LLM caches behind LlmResponseCache.                  │
│  L1 memory + L2 public.llm_cache (namespace-scoped).                         │
│  Keys: {mode}:{cache_type}:{hash}-cache  (LR shape + SPEC-091 suffix).       │
│  Product default ON (EDGEQUAKE_LLM_CACHE). Acc: set =0 / c1cold honesty.     │
│  Query hash includes full RAG prompt (safer than LR context-free keys).      │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Start here

1. [00-why.md](./00-why.md) — product UX vs Acc fairness  
2. [00-first-principles.md](./00-first-principles.md) — LAW-C1…C8  
3. [01-finding-register.md](./01-finding-register.md) — gaps  
4. [02-cross-ref-matrix.md](./02-cross-ref-matrix.md) — code ↔ law ↔ test  
5. [03-implementation-roadmap.md](./03-implementation-roadmap.md)  
6. [04-e2e-test-matrix.md](./04-e2e-test-matrix.md)  
7. [05-edge-cases.md](./05-edge-cases.md)

## Locked decisions

| Decision | Choice |
|----------|--------|
| Master switch | `EDGEQUAKE_LLM_CACHE` default **on** |
| Persistence | `public.llm_cache` (no workspace column) |
| Query key body | Full built RAG prompt (context-inclusive) |
| Extract/summary | Out of v1 (ingest/MM paths stay) |
| Acc fairness | Pin `EDGEQUAKE_LLM_CACHE=0` on Acc backends |
| Provider KV (SPEC-126) | Separate switch; Acc **leaves on** |

## Surfaces

| Surface | Env / API |
|---------|-----------|
| Master | `EDGEQUAKE_LLM_CACHE` |
| Keyword override | `EDGEQUAKE_KEYWORD_CACHE` |
| Answer override | `EDGEQUAKE_QUERY_ANSWER_CACHE` |
| Stats | `keyword_cache_hit`, `answer_cache_hit` |
| Proof | `make spec103-llm-cache-proof` |

## Verify

```bash
cd edgequake && cargo test -p edgequake-query --lib cache::
cd edgequake && cargo test -p edgequake-query --test contract_spec103_llm_cache
make spec103-llm-cache-proof   # postgres subset when DATABASE_URL set
```
