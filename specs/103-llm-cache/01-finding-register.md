# SPEC-103 — Finding Register

| ID | Finding | Severity | Law |
|----|---------|----------|-----|
| F-103-01 | Keyword cache is process-local LRU only — dies on restart / multi-replica | High | C4, C5 |
| F-103-02 | Answer cache default off + memory-only — not LR product parity | High | C6 |
| F-103-03 | No unified `LlmResponseCache` trait; keyword vs answer duplicated | Med | DRY |
| F-103-04 | `public.llm_cache` ready but unused by query keyword/answer paths | High | C4 |
| F-103-05 | No master `enable_llm_cache` switch for Acc honesty | Med | C6, C7 |
| F-103-06 | `keyword_cache_hit` not exposed on query stats | Med | C8 |
| F-103-07 | LR `openai_complete_if_cache` naming trap — real path is handle_cache | Info | docs |
| F-103-08 | LR query keys omit retrieved context → stale risk; EQ must not copy | High | C3 |
