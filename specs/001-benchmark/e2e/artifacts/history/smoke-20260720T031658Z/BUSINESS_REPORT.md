# EdgeQuake vs LightRAG — Business Performance Report

**Generated:** 2026-07-20T03:16:58Z  
**Task:** GraphRAG-Bench Acc dual-SUT (July 2026 fair pins)  
**Profile:** `S0_p2b_l2_sources_union_v1_lrlike_arms_v2`  
**Valid run:** `True`

## Verdict

```text
  STATISTICAL TIE on answer quality
  Acc   EdgeQuake 0.726  ·  LightRAG 0.775  ·  Δ -0.049
  Δ Acc 95% CI: [-0.114, +0.020] (n=40) — includes 0 ⇒ tie
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
| Answer quality (Acc) | Are answers roughly as good? | 0.726 | 0.775 | Tie (CI) |
| Evidence coverage | Did we find the right sources? | 0.929 | 0.966 | LightRAG |
| Context cleanliness | Is the prompt low-noise? | 0.487 | 0.544 | LightRAG |
| Speed (query p50) | Time to answer (ms) | 11580 | 1972 | LightRAG |

- **EQ/LR p50 ratio:** 5.872× (product SLO target ≤ 1.5×)

## By question type (Acc)

| User need | EdgeQuake | LightRAG | Who leads |
|-----------|-----------|----------|-----------|
| Fact lookup | 0.664 | 0.710 | LightRAG |
| Multi-hop reasoning | 0.687 | 0.833 | LightRAG |
| Summarization | 0.776 | 0.836 | LightRAG |
| Creative / open-ended | 0.776 | 0.721 | EdgeQuake |

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
- **Archive:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T031658Z`
- **Technical SUMMARY:** same folder / archive `SUMMARY.md`
- **Static business brief:** `specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md`
- **Acc honesty close:** `specs/001-benchmark/001-edgquake-improvements/018-e4-acc-tie-close.md`
