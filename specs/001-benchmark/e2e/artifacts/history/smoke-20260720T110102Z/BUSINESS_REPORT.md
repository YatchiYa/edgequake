# EdgeQuake vs LightRAG — Business Performance Report

**Generated:** 2026-07-20T11:01:02Z  
**Task:** GraphRAG-Bench Acc dual-SUT (July 2026 fair pins)  
**Profile:** `A1FPSEL_p2b_rr_cer_fact_protect_topic_admit_v1_lrlike_arms_v2`  
**Valid run:** `True`

## Verdict

```text
  STATISTICAL TIE on answer quality
  Acc   EdgeQuake 0.719  ·  LightRAG 0.783  ·  Δ -0.063
  Δ Acc 95% CI: [-0.158, +0.033] (n=40) — includes 0 ⇒ tie
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
| Answer quality (Acc) | Are answers roughly as good? | 0.719 | 0.783 | Tie (CI) |
| Evidence coverage | Did we find the right sources? | 0.911 | 0.962 | LightRAG |
| Context cleanliness | Is the prompt low-noise? | 0.525 | 0.525 | Tie |
| Speed (query p50) | Time to answer (ms) | 7548 | 1660 | LightRAG |

- **EQ/LR p50 ratio:** 4.547× (product SLO target ≤ 1.5×)

## By question type (Acc)

| User need | EdgeQuake | LightRAG | Who leads |
|-----------|-----------|----------|-----------|
| Fact lookup | 0.718 | 0.722 | LightRAG |
| Multi-hop reasoning | 0.681 | 0.835 | LightRAG |
| Summarization | 0.771 | 0.855 | LightRAG |
| Creative / open-ended | 0.708 | 0.718 | LightRAG |

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
- **Archive:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T110102Z`
- **Technical SUMMARY:** same folder / archive `SUMMARY.md`
- **Static business brief:** `specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md`
- **Acc honesty close:** `specs/001-benchmark/001-edgquake-improvements/018-e4-acc-tie-close.md`
