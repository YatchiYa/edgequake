# 064 — Product latency polish: TTFT · keyword/answer cache · batch embed

**Status:** Done · warm peer measured · **not Acc Beat fishing** · Acc Fact / cold peers unchanged  
**Date:** 2026-07-21 (evidence refresh 2026-08-15)  
**Cross-ref:** [061](./061-lightrag-law-first-principles-eq.md) Ideas B/C/D · [063](./063-why-lightrag-faster-cache-fairness.md) cold ≈1.01× · **[SPEC-103](../../../103-llm-cache/)** supersedes answer default-off for product (master `EDGEQUAKE_LLM_CACHE` ON; Acc still pins `0`) · peer [`EQ_LLM_CACHE_WARM_v1`](../e2e/artifacts/publish/peers/EQ_LLM_CACHE_WARM_v1/)

---

## 1. First principles

| Law | Detail |
|-----|--------|
| UX ≠ completion | Users feel **TTFT** (blank→first token), not full answer wall ([AWS AGENTPERF02-BP04](https://docs.aws.amazon.com/wellarchitected/latest/agentic-ai-lens/agentperf02-bp04.html): track pipeline TTFT vs model TTFT separately). |
| Cache identity | LR Acc looked fast because of keyword+answer LLM cache ([063](./063-why-lightrag-faster-cache-fairness.md)). EQ product may offer the same — **labeled**, default **off** on Acc fairness. |
| Embed RTT | One network round-trip per unique text; batch `embed` must share the `embed_one` cache. |
| Workspace inject | Acc Mix always passes a fresh workspace embedder. Same `(name, model, dim)` must reuse the engine LRU — otherwise every query pays mistral-embed (~2s) even when texts are warm. |
| Acc STOP | No Soft Mix Acc; no Acc Beat from warm latency. Cold peer stays `c1cold`. |

```text
- [x] Step 1: FP hub (this doc)
- [x] Step 2: TTFT on stream Done stats (`ttft_ms` + `ux_ttft_ms`)
- [x] Step 3: Answer cache opt-in + keyword cache disable env
- [x] Step 4: Cache-aware batch embed
- [x] Step 5: Unit tests + wire docs / .env.example
- [x] Step 6: Workspace inject → engine embed LRU (Acc Mix path)
- [x] Step 7: Labeled warm peer EQ_LLM_CACHE_WARM_v1 (EQ 82 / LR 993 ms)
```

---

## 2. Deliverables

| Feature | Env / surface | Default |
|---------|---------------|---------|
| **TTFT** | `QueryStreamStats.ttft_ms` (gen→first token) + `ux_ttft_ms` (request→first token) | Always measured on stream |
| **Keyword cache** | `EDGEQUAKE_KEYWORD_CACHE=0` / master `EDGEQUAKE_LLM_CACHE=0` | **on** (product; Acc pins master off) |
| **Answer cache** | unset follows master; `EDGEQUAKE_QUERY_ANSWER_CACHE=0` fine override | **on** with master (SPEC-103); Acc pins master `0` |
| **Batch embed cache** | transparent in `CachingEmbeddingProvider::embed` | on when embed cache wrapped |
| **Workspace embed coerce** | `QueryEngine::cached_embedding_for` on Mix inject | always (same identity → engine LRU) |

### Code map

| Piece | Location |
|-------|----------|
| Unified LLM cache | `edgequake-query/src/cache/llm_response_cache.rs` → `public.llm_cache` (SPEC-103) |
| Answer cache | `answer_cache.rs` (L1 helper) · `QueryEngine::with_answer_cache_from_env` |
| Keyword cache pin | `keywords/keyword_mode.rs` → `keyword_cache_enabled()` (honors master) |
| Batch embed | `cache/embedding_cache.rs` · per-text hit/miss + shared keys with `embed_one` |
| Workspace inject LRU | `engine_impl/mod.rs` `cached_embedding_for` · `query_workspace.rs` / `query_stream.rs` |
| Stream TTFT | `edgequake-api/.../query_stream.rs` Done event |
| Bootstrap | `build_production_query_engine` → `.with_answer_cache_from_env()` |
| Warm peer | `make bench001-medical-mid-eq-llm-cache-warm` → `EQ_LLM_CACHE_WARM_v1` |

---

## 3. Success (product, not Acc Acc)

1. Stream Done event reports `ttft_ms` and `ux_ttft_ms`.  
2. With master cache on (default) or `EDGEQUAKE_QUERY_ANSWER_CACHE=1`, repeated identical Mix query hits cache (`answer_cache_hit`, generation ≈0). See SPEC-103.  
3. Batch `embed([a,b])` after `embed_one(a)` hits cache for `a` (unit ✓).  
4. Acc Mix workspace inject reuses engine embed LRU when identity matches (e2e ✓).  
5. Labeled warm peer: EQ p50 **82 ms** vs LR **993 ms** (0.083×); stages kw/embed/gen **0**.  
6. No Acc Fact peer promote; `c1cold` remains fair cold latency truth.

---

## 4. Non-goals

- Acc Beat / Soft Mix  
- Promoting warm peer over `c1cold` as cold engine SSOT  
- Changing Acc default answer cache on  
- Overwriting Acc `publish/latest`

---

## 5. How to demo (product only)

```bash
# Labeled warm peer (NOT Acc Acc):
make bench001-medical-mid-eq-llm-cache-warm
# → specs/001-benchmark/e2e/artifacts/publish/peers/EQ_LLM_CACHE_WARM_v1/

# Fair Acc latency truth stays:
make bench001-c1cold   # BENCH001_LR_ENABLE_LLM_CACHE=0 · EQ Acc pins EDGEQUAKE_LLM_CACHE=0
```

---

## 6. Verification

```bash
cd edgequake && cargo test -p edgequake-query --test e2e_spec103_warm_embed_batch --test contract_embedding_cache
# warm Mix + workspace inject embed LRU
```
