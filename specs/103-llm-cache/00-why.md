# SPEC-103 — Why

## Five WHYs

1. **Why does LightRAG look ~4× faster on warm Acc?**  
   Because `enable_llm_cache=True` serves keyword + answer from disk KV — not a faster Mix engine ([063](../001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md)).

2. **Why does EdgeQuake still pay keyword+generate every query?**  
   Keyword cache is process-local; answer cache is opt-in off and also process-local (064).

3. **Why not just flip `EDGEQUAKE_QUERY_ANSWER_CACHE=1`?**  
   Memory-only does not survive restart or multi-replica; there is no unified LR-shaped key/envelope; Acc needs a single honest off switch.

4. **Why reuse `public.llm_cache`?**  
   SPEC-091 already owns typed cache SSOT + namespace scope. Reinventing a table violates DRY and the migration-owned schema law.

5. **Why not copy LR’s context-free query keys?**  
   LR can return a cached answer after the graph changes. EQ hashes the full RAG prompt so context drift invalidates (LAW-C3 divergence).

## Causal chain

```text
Warm Acc LR fast
  → keyword hit + query hit (enable_llm_cache)
  → EQ Acc still cold LLM
  → unfair latency claim
Product wants LR UX
  → durable keyword + answer cache
  → Acc pin cache OFF (c1cold honesty)
```
