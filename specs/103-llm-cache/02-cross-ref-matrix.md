# SPEC-103 — Cross-Ref Matrix

| Code / artifact | Law | Finding | Test ID |
|-----------------|-----|---------|---------|
| `cache::llm_response_cache::llm_cache_key` | C2 | F-103-03 | `spec103_key_format_and_hash_pins` |
| `hash_keyword_args` / `hash_query_prompt` | C3 | F-103-08 | `spec103_key_format_and_hash_pins` |
| `resolve_llm_cache_flags` | C6, C7 | F-103-05 | `spec103_master_off_disables_both` |
| `TieredLlmResponseCache` | C4, C5 | F-103-01/04 | `spec103_tiered_memory_then_postgres` |
| `CachedKeywordExtractor` → LlmResponseCache | C1, C8 | F-103-01 | `spec103_keyword_hit_skips_llm` |
| `query_pipeline` answer path | C3, C8 | F-103-02 | `spec103_query_answer_hit_skips_generate` |
| `query_stream` answer write/hit | C5, C8 | F-103-02 | `spec103_query_answer_hit_skips_generate` |
| `QueryStats.keyword_cache_hit` | C8 | F-103-06 | `spec103_keyword_hit_skips_llm` |
| `build_production_query_engine` wiring | C4, C6 | F-103-04 | `spec103_persist_across_engine_rebuild` |
| Acc backend `EDGEQUAKE_LLM_CACHE=0` | C7 | F-103-05 | Acc pin / doctor docs |
| Vision / empty context bypass | C1 | — | `spec103_vision_or_empty_context_bypass` |
| `public.llm_cache` / SPEC-091 scope | C4 | F-103-04 | `contract_spec091_llm_cache_scope` (peer) |

## External refs

- LightRAG: `handle_cache` / `save_to_cache` / `generate_cache_key`  
- EQ multimodal template: `edgequake-api/.../multimodal/cache.rs`  
- Acc cold peer: `make bench001-c1cold`  
