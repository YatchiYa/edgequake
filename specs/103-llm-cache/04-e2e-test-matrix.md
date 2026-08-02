# SPEC-103 — E2E Test Matrix

| Test ID | Kind | Gate |
|---------|------|------|
| `spec103_key_format_and_hash_pins` | unit | Key shape; pin sensitivity; `\x1e` delimiter |
| `spec103_tiered_memory_then_postgres` | unit/contract | L1 clear still hits L2 |
| `spec103_keyword_hit_skips_llm` | query e2e (mock) | 2nd extract → hit, counting LLM not called |
| `spec103_query_answer_hit_skips_generate` | query e2e | 2nd Mix → `answer_cache_hit` |
| `spec103_master_off_disables_both` | query e2e | Master off → no hits |
| `spec103_persist_across_engine_rebuild` | postgres e2e | Rebuild engine → hit |
| `spec103_vision_or_empty_context_bypass` | unit | No empty-answer poison |

## Commands

```bash
cd edgequake && cargo test -p edgequake-query --lib cache::llm_response_cache
cd edgequake && cargo test -p edgequake-query --test contract_spec103_llm_cache
# postgres (optional):
DATABASE_URL=... cargo test -p edgequake-query --test e2e_spec103_llm_cache_persist -- --ignored
make spec103-llm-cache-proof
```
