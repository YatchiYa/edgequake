# 064 — Product latency polish: TTFT · keyword/answer cache · batch embed

**Status:** Done · **not Acc Beat fishing** · Acc Fact / cold peers unchanged  
**Date:** 2026-07-21  
<<<<<<< HEAD
**Cross-ref:** [061](./061-lightrag-law-first-principles-eq.md) Ideas B/C/D · [063](./063-why-lightrag-faster-cache-fairness.md) cold ≈1.01×
=======
**Cross-ref:** [061](./061-lightrag-law-first-principles-eq.md) Ideas B/C/D · [063](./063-why-lightrag-faster-cache-fairness.md) cold ≈1.01× · **[SPEC-103](../../../103-llm-cache/)** supersedes answer default-off for product (master `EDGEQUAKE_LLM_CACHE` ON; Acc still pins `0`)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

---

## 1. First principles

| Law | Detail |
|-----|--------|
| UX ≠ completion | Users feel **TTFT** (blank→first token), not full answer wall ([AWS AGENTPERF02-BP04](https://docs.aws.amazon.com/wellarchitected/latest/agentic-ai-lens/agentperf02-bp04.html): track pipeline TTFT vs model TTFT separately). |
| Cache identity | LR Acc looked fast because of keyword+answer LLM cache ([063](./063-why-lightrag-faster-cache-fairness.md)). EQ product may offer the same — **labeled**, default **off** on Acc fairness. |
| Embed RTT | One network round-trip per unique text; batch `embed` must share the `embed_one` cache. |
| Acc STOP | No Soft Mix Acc; no warm-LR Beat claims. Cold peer stays `c1cold`. |

```text
- [x] Step 1: FP hub (this doc)
- [x] Step 2: TTFT on stream Done stats (`ttft_ms` + `ux_ttft_ms`)
- [x] Step 3: Answer cache opt-in + keyword cache disable env
- [x] Step 4: Cache-aware batch embed
- [x] Step 5: Unit tests + wire docs / .env.example
```

---

## 2. Deliverables

| Feature | Env / surface | Default |
|---------|---------------|---------|
| **TTFT** | `QueryStreamStats.ttft_ms` (gen→first token) + `ux_ttft_ms` (request→first token) | Always measured on stream |
<<<<<<< HEAD
| **Keyword cache** | `EDGEQUAKE_KEYWORD_CACHE=0` to disable (already on) | **on** |
| **Answer cache** | `EDGEQUAKE_QUERY_ANSWER_CACHE=1` | **off** (Acc-safe) |
=======
| **Keyword cache** | `EDGEQUAKE_KEYWORD_CACHE=0` / master `EDGEQUAKE_LLM_CACHE=0` | **on** (product; Acc pins master off) |
| **Answer cache** | unset follows master; `EDGEQUAKE_QUERY_ANSWER_CACHE=0` fine override | **on** with master (SPEC-103); Acc pins master `0` |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
| **Batch embed cache** | transparent in `CachingEmbeddingProvider::embed` | on when embed cache wrapped |

### Code map

| Piece | Location |
|-------|----------|
<<<<<<< HEAD
| Answer cache | `edgequake-query/src/cache/answer_cache.rs` · `QueryEngine::with_answer_cache_from_env` |
| Keyword cache pin | `keywords/keyword_mode.rs` → `keyword_cache_enabled()` |
=======
| Unified LLM cache | `edgequake-query/src/cache/llm_response_cache.rs` → `public.llm_cache` (SPEC-103) |
| Answer cache | `answer_cache.rs` (L1 helper) · `QueryEngine::with_answer_cache_from_env` |
| Keyword cache pin | `keywords/keyword_mode.rs` → `keyword_cache_enabled()` (honors master) |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
| Batch embed | `cache/embedding_cache.rs` · per-text hit/miss + shared keys with `embed_one` |
| Stream TTFT | `edgequake-api/.../query_stream.rs` Done event |
| Bootstrap | `build_production_query_engine` → `.with_answer_cache_from_env()` |

---

## 3. Success (product, not Acc Acc)

1. Stream Done event reports `ttft_ms` and `ux_ttft_ms`.  
<<<<<<< HEAD
2. With `EDGEQUAKE_QUERY_ANSWER_CACHE=1`, repeated identical Mix query hits cache (`answer_cache_hit`, generation ≈0).  
=======
2. With master cache on (default) or `EDGEQUAKE_QUERY_ANSWER_CACHE=1`, repeated identical Mix query hits cache (`answer_cache_hit`, generation ≈0). See SPEC-103.  
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
3. Batch `embed([a,b])` after `embed_one(a)` hits cache for `a` (unit ✓).  
4. No Acc Fact peer promote; `c1cold` remains fair latency truth.

---

## 4. Non-goals

- Acc Beat / Soft Mix  
- Claiming ≤1.5× from warm LR archives  
- Changing Acc default answer cache on  
- Promoting any warm-LR latency peer over `c1cold`

---

## 5. How to demo (product only)

```bash
<<<<<<< HEAD
# Answer cache demo (NOT Acc Acc peer)
export EDGEQUAKE_QUERY_ANSWER_CACHE=1
# Restart backend; repeat identical Mix query — 2nd call should show answer_cache_hit.

# Fair Acc latency truth stays:
make bench001-c1cold   # BENCH001_LR_ENABLE_LLM_CACHE=0
=======
# Answer cache demo (NOT Acc Acc peer) — product default is master ON (SPEC-103)
export EDGEQUAKE_LLM_CACHE=1
# Restart backend; repeat identical Mix query — 2nd call should show answer_cache_hit.

# Fair Acc latency truth stays:
make bench001-c1cold   # BENCH001_LR_ENABLE_LLM_CACHE=0 · EQ Acc pins EDGEQUAKE_LLM_CACHE=0
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
```

---

## 6. Verification

```bash
cd edgequake && cargo test -p edgequake-query --lib cache::
# 12 passed (answer_cache + embedding_cache + keyword cache)
```
