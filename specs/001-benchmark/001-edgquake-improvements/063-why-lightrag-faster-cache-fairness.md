# 063 — Why is LightRAG faster on Acc? (code + LLM assessment)

**Status:** Diagnosis ✓ · cold-fair pack `c1cold` · Acc Fact peer unchanged  
**Date:** 2026-07-21  
<<<<<<< HEAD
**Cross-ref:** [061](./061-lightrag-law-first-principles-eq.md) · [059](./059-c1b-latency-ceiling-keyword-embed.md) · [062](./062-c1e-fast-keyword-llm.md)  
=======
**Cross-ref:** [061](./061-lightrag-law-first-principles-eq.md) · [059](./059-c1b-latency-ceiling-keyword-embed.md) · [062](./062-c1e-fast-keyword-llm.md) · **[SPEC-103](../../../103-llm-cache/)** (EQ durable LR-parity cache)  
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
**Law sources:** LightRAG `enable_llm_cache=True` · `tools/bench001/bench001/lightrag_runner.py`

---

## 0. Todo

```text
- [x] Step 1: Code-explore Acc LR vs EQ LLM / rerank / concurrency
- [x] Step 2: Verify warm LLM cache hits on Acc smoke questions
- [x] Step 3: Harness pin BENCH001_LR_ENABLE_LLM_CACHE + pack c1cold
- [x] Step 4: Measure c1cold — archive T022103Z · EQ/LR p50 ≈ 1.01× PASS
```

---

## 1. First principles question

| Q | Naive claim | Law |
|---|-------------|-----|
| Different LLM? | “LR uses a faster KEYWORD model” | **False on Acc.** Same `mistral-small-latest` for KEYWORD + QUERY (`lightrag_runner` single `llm_model_func`; no `KEYWORD_LLM_*`). |
| Faster engine? | “LR Mix is inherently 4× faster” | **Not proven** from warm archives. EQ generate alone (C1b **2421** ms) **>** LR full query p50 (**1481** ms). |
| Fair latency? | EQ/LR p50 from Acc `--query-only` | **Unfair** if LR keyword+answer cache is warm and EQ has no answer cache. |

---

## 2. Code exploration (Acc dual-SUT)

### Same models

| Side | LLM | Embed | Rerank |
|------|-----|-------|--------|
| EQ Acc | `mistral-small-latest` | `mistral-embed` | BM25 on C1b/c1cold (~9 ms) |
| LR Acc | **same** `mistral-small-latest` | **same** `mistral-embed` | **`enable_rerank=false`** (fair pin) |

Product LightRAG docs recommend KEYWORD≠QUERY; **Acc harness does not configure that.**

### Dominant Acc confound — warm LLM cache

```text
LightRAG(enable_llm_cache=True)  # default in lightrag.py
working_dir = ~/.cache/edgequake/bench001/lightrag/smoke/
kv_store_llm_response_cache.json  (~11 MB)
  → keywords: 88 entries · query: 224 entries
  → Acc smoke n=40: 40/40 keyword hits · 40/40 query hits  (verified 2026-07-21)
```

Warm LR wall ≈ **embed + retrieve** (cache skips keyword LLM + answer LLM).  
EQ Acc still pays **keyword LLM + generate** every query.

### Secondary (not the 4× gap)

| Factor | Evidence | Rank |
|--------|----------|------|
| Concurrency EQ 4 vs LR ≤2 | scorecard / Makefile | MED |
| LR 1-batch embed vs EQ embed_one+follow-up | operate.py vs query_pipeline | MED (cold path) |
| Larger EQ context → slower generate | pred context chars | MED |
| Parallel Mix arms | EQ better here | does **not** explain LR win |
| Different KEYWORD model | Acc LR: none | **ruled out** |

---

## 3. Physics check (C1b warm archive)

| Metric | Value |
|--------|------:|
| EQ keyword p50 | 1782 |
| EQ generate p50 | 2421 |
| EQ total p50 | 5791 |
| LR total p50 | 1481 |
| EQ/LR (published) | 3.91× |

**Law:** If LR ran the same two mistral-small chat calls cold, LR p50 must rise toward EQ’s keyword+generate band (minus EQ-only overhead). Warm ~1.5s is **not** a product engine win claim.

---

## 4. Fairness fix (harness)

| Piece | Change |
|-------|--------|
| `lightrag_runner.py` | `enable_llm_cache=` from `BENCH001_LR_ENABLE_LLM_CACHE` (default `1`) |
| `fair_pins` | record `lr_enable_llm_cache` |
| Pack **`c1cold`** | C1b pins + `BENCH001_LR_ENABLE_LLM_CACHE=0` |
| Make | `make bench001-c1cold` |

Acc Acc / Acc Fact peer keep cache **on** (Acc quality unchanged; latency claims must use cold peer).

```bash
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
make bench001-c1cold
```

---

## 5. Measured cold peer (`T022103Z`)

| Metric | Warm C1b | **Cold c1cold** |
|--------|---------:|----------------:|
| EQ query p50 | 5791 | **5891** |
| LR query p50 | **1481** | **5817** |
| **EQ/LR p50** | **3.91×** | **1.013×** |
| SLO ≤1.5× | FAIL | **PASS** |
| `lr_enable_llm_cache` | true (default) | **false** |
| EQ stages (cold) | — | keyword 1678 · embed 2275 · retrieve 843 · rerank 11 · generate 2311 |

Log confirmed: `LR enable_llm_cache=False`. Same models both sides. Acc Δ still a statistical tie (EQ 0.731 / LR 0.760).

**Verdict:** The Acc “LR is ~4× faster” headline was **warm LLM cache**, not a different KEYWORD model and not a 4× engine gap. Under fair cold Mix, EQ ≈ LR wall time.

---

## 6. What this means for Horizon C

1. **Retire** warm-archive EQ/LR p50 as latency SLO evidence (label as cache-aided).  
2. Publishable latency peer = **`c1cold`** (or any pack with `lr_enable_llm_cache=false`).  
3. Remaining EQ product wins (vs LR cold parity): TTFT/stream UX, keyword/answer **product** cache, 1-batch embed — optional polish, not Acc Beat fishing.  
4. **c1e** still valid as EQ KEYWORD-role wiring; it was never going to beat warm LR answer cache.

---

## 7. Binding claims

| Allowed | Forbidden |
|---------|-----------|
| “Warm Acc LR p50 is cache-aided (~1.5s)” | “EQ is 4× slower than LR under fair Acc latency” from warm archives |
| “Cold Acc latency peer ≈ 1.0× (T022103Z)” | “LR Acc is faster because of KEYWORD role model” |
| “Acc uses same mistral-small both sides” | Acc Beat / Acc Fact claims from `c1cold` |
