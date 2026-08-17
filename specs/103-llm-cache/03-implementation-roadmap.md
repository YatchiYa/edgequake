# SPEC-103 — Implementation Roadmap

## Wave 1 — Core module

- [x] Spec pack  
- [x] `llm_response_cache.rs`: keys, hashes, flags, trait, memory + tiered  
- [x] Unit tests for keys/flags  

## Wave 2 — Wire query path

- [x] Keyword extractor uses durable cache when KV present  
- [x] Answer path uses same trait (pipeline + stream)  
- [x] `keyword_cache_hit` on `QueryStats`  
- [x] Bootstrap: master ON ⇒ answer on unless override  

## Wave 3 — Acc + docs + proof

- [x] Acc backend / `start_acc_backend` pin `EDGEQUAKE_LLM_CACHE=0`  
- [x] `.env.example` + AGENTS + 063/064 cross-links  
- [x] Contract/e2e tests + `make spec103-llm-cache-proof`  

## Definition of Done

1. Repeated Mix query with cache on → `answer_cache_hit` after first generate.  
2. Keyword extract reused → `keyword_cache_hit`.  
3. Engine rebuild with same namespace → L2 hit.  
4. `EDGEQUAKE_LLM_CACHE=0` → no hits.  
5. Acc Acc backends export cache off.  
