# EdgeQuake vs LightRAG — Business Performance Report

**Generated:** 2026-07-20T10:00:53Z  
**Task:** GraphRAG-Bench Acc dual-SUT (July 2026 fair pins)  
**Profile:** `A1FPLR_p2b_rr_cer_fact_protect_lr_budget_v1_lrlike_arms_v2`  
**Valid run:** `True`

## Verdict

```text
  STATISTICAL TIE on answer quality
  Acc   EdgeQuake 0.738  ·  LightRAG 0.784  ·  Δ -0.046
  Δ Acc 95% CI: [-0.118, +0.036] (n=40) — includes 0 ⇒ tie
```

## What we tested

- Same GraphRAG-Bench medical questions (n=40) for both systems
- Same generator/judge stack: `mistral/mistral-small-latest` · embeddings `mistral/mistral-embed`
- Same Mix mode, matched top-k / chunk size, official `generation_eval` Acc
- Fairness: Mix arms always on, RRF fusion, chunk 1200/100, related_chunk=5
- **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE)

## Scorecard for decisions

| Layer | Plain meaning | EdgeQuake | LightRAG | Winner |
|-------|---------------|-----------|----------|--------|
| Answer quality (Acc) | Are answers roughly as good? | 0.738 | 0.784 | Tie (CI) |
| Evidence coverage | Did we find the right sources? | 0.918 | 0.962 | LightRAG |
| Context cleanliness | Is the prompt low-noise? | 0.519 | 0.525 | LightRAG |
| Speed (query p50) | Time to answer (ms) | 6707 | 1325 | LightRAG |

- **EQ/LR p50 ratio:** 5.062× (product SLO target ≤ 1.5×)

## By question type (Acc)

| User need | EdgeQuake | LightRAG | Who leads |
|-----------|-----------|----------|-----------|
| Fact lookup | 0.706 | 0.772 | LightRAG |
| Multi-hop reasoning | 0.717 | 0.838 | LightRAG |
| Summarization | 0.778 | 0.859 | LightRAG |
| Creative / open-ended | 0.752 | 0.669 | EdgeQuake |

## July 2026 landscape

On this fair Acc head-to-head, EdgeQuake is a **LightRAG-class GraphRAG peer** when Acc is statistically tied. In the GraphRAG-Bench literature (ICLR 2026), **HippoRAG2-class** systems define the aspirational **retrieval SOTA** (high evidence recall **and** high context relevancy with compact prompts). Absolute Acc numbers from the academic paper use different models and are **not** directly comparable to these Mistral Acc pins.

## Allowed / forbidden external claims

| Allowed | Forbidden |
|---------|-----------|
| “Peer / statistical tie with LightRAG on Acc under fair pins” (or “point estimate ahead” only if CI excludes 0) | “Beats LightRAG” / “wins Acc” / “#1 GraphRAG-Bench” / “SOTA RAG” without CI excluding 0 and L2 gates |
| “Peer GraphRAG with production stack (Postgres, API, PDF pipeline)” | “#1 on GraphRAG-Bench” without matching paper protocol |
| “Actively closing retrieval noise / multi-hop / latency gaps” | Silent Acc headline = CE+protect without promotion gate |

## How to reproduce

```bash
make bench                 # cold full Acc (n=40) + this publish pack
make bench-warm            # query-only (auto latest warm EQ workspace)
```

- Pins: Mix arms on · RRF · chunk 1200/100 · top-k 30 · `mistral/mistral-small-latest` + `mistral/mistral-embed`

## Pointers

- **This publish pack:** `specs/001-benchmark/e2e/artifacts/publish/latest/`
- **Archive:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T100053Z`
- **Technical SUMMARY:** same folder / archive `SUMMARY.md`
- **Static business brief:** `specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md`
- **Acc honesty close:** `specs/001-benchmark/001-edgquake-improvements/018-e4-acc-tie-close.md`
