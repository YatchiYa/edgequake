# EdgeQuake vs LightRAG — Labeled Warm Latency Peer

**Generated:** 2026-08-15T13:20:34Z  
**Peer:** `EQ_LLM_CACHE_WARM_v1` (**labeled** — not Acc headline)  
**Task:** GraphRAG-Bench Mix dual-SUT · medical-mid n=200  
**Profile:** `ACC_E2OCC_086_v1_lrlike_arms_v2`  
**Valid run:** `True`

## One-screen first principles

```text
  Peer     EQ_LLM_CACHE_WARM_v1  (product warm latency · matched LLM cache)
  Sample   n=200 medical-mid · workspace 23b09c73-… · query-only fill+warm
  Latency  EQ p50 82 ms · LR p50 993 ms · ratio 0.083×
  Stages   EQ kw=0 embed=0 retrieve=56 gen=0  (retrieve-only warm path)
  Acc      EQ 0.792 · LR 0.773 · Δ CI [-0.035, +0.058] ⇒ statistical TIE
  Verdict  WARM LATENCY WIN (labeled) · Acc still NOT Beat
```

## Verdict

```text
  Warm Mix wall: EdgeQuake BEATS LightRAG (0.083×)
  Acc quality:   STATISTICAL TIE — do not claim Acc Beat
  Acc SSOT:      unchanged at publish/latest (T110218Z)
```

## What we tested

- Same GraphRAG-Bench medical questions (n=200) for both systems
- Same Mix mode · Mistral Small + `mistral-embed` · official `generation_eval`
- **EQ** `EDGEQUAKE_LLM_CACHE=1` (keywords + answer L1/L2) + engine embed LRU on workspace inject
- **LR** `enable_llm_cache=True` (default Acc peer fairness for warm — see [063](../../../../../001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md))
- Pass 1 fill → Pass 2 warm measure · `SKIP publish/latest`

## Scorecard for decisions

| Layer | Plain meaning | EdgeQuake | LightRAG | Winner |
|-------|---------------|-----------|----------|--------|
| Warm speed (query p50) | Time to answer when caches are warm | **82 ms** | 993 ms | **EdgeQuake** |
| EQ stage honesty | Where the wall goes | retrieve 56 · others 0 | — | Retrieve-only |
| Answer quality (Acc) | Are answers as good? | 0.792 | 0.773 | Tie (CI) |
| Evidence coverage | Right sources? | 0.934 | 0.958 | LightRAG |
| Context cleanliness | Prompt noise | 0.470 | 0.506 | LightRAG |

- **EQ/LR p50 ratio:** **0.083×** (product SLO ≤1.5× **PASS** on this labeled warm peer)

## First-principles fix that unlocked the win

Acc `/query` always injects a workspace embedding provider. Before this peer, that Arc bypassed the engine embed LRU → every warm Mix still paid ~2s `mistral-embed` RTT. Same `(name, model, dim)` now reuses the engine cache (064 / SPEC-103). Proof: warm embed p50 **0**.

## Allowed / forbidden claims

| Allowed | Forbidden |
|---------|-----------|
| “Warm Mix with LLM+embed cache: EQ p50 82 ms vs LR 993 ms (0.083×)” | “Acc Beat” / “beats LightRAG on Acc” |
| “Product warm latency peer `EQ_LLM_CACHE_WARM_v1`” | Replacing Acc `publish/latest` with this pack |
| “Fair cold latency remains `C1COLD_v1` ~1.02×” | Claiming this ratio as cold/engine-only truth |

## How to reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>   # Acc full-corpus workspace
make bench001-medical-mid-eq-llm-cache-warm             # fill + warm → this peer
```

Fair cold latency (unchanged Acc law):

```bash
make bench001-c1cold   # BENCH001_LR_ENABLE_LLM_CACHE=0 · EQ Acc pins CACHE=0
```

## Pointers

- **This labeled peer:** `specs/001-benchmark/e2e/artifacts/publish/peers/EQ_LLM_CACHE_WARM_v1/`
- **Archive:** [`medical-mid-20260815T132034Z`](../../../history/medical-mid-20260815T132034Z/)
- **Acc SSOT (unchanged):** [`publish/latest/`](../../latest/) · [`medical-mid-20260815T110218Z`](../../../history/medical-mid-20260815T110218Z/)
- **Cold latency SSOT:** peer [`C1COLD_v1`](../C1COLD_v1/)
- **Comparison doc:** [`docs/comparisons/eq-vs-lightrag-acc-bench.md`](../../../../../../../docs/comparisons/eq-vs-lightrag-acc-bench.md)
- **064 product cache law:** [`064-product-ttft-cache-batch-embed.md`](../../../../../001-edgquake-improvements/064-product-ttft-cache-batch-embed.md)
