# EdgeQuake vs LightRAG — Business Performance Report

**Generated:** 2026-07-23T01:26:53Z  
**Task:** GraphRAG-Bench Acc dual-SUT (July 2026 fair pins)  
**Profile:** `LR_RELSEL_FACT_L2_v1_lrlike_arms_v2`  
**Fixture:** `smoke_question_ids_v1` (n=40)  
**Valid run:** `True`

## One-screen first principles

```text
  Task     GraphRAG-Bench/EQ-vs-LR  (same corpus · questions · judge · Mix↔Mix)
  Sample   n=40 (smoke gate — not a release publish claim; use medical-mid n=200)
  Acc      EQ 0.709 · LR 0.775 · Δ -0.066
  Δ Acc CI [-0.164, +0.031] (n=40) — includes 0 ⇒ tie
  L2       evidence recall EQ 0.934 / LR 0.964 · ctx_rel EQ 0.475 / LR 0.537
  Latency  query p50 EQ 4582 ms / LR 1092 ms
  Verdict  STATISTICAL TIE on answer quality
```

## Verdict

```text
  STATISTICAL TIE on answer quality
  Acc   EdgeQuake 0.709  ·  LightRAG 0.775  ·  Δ -0.066
  Δ Acc 95% CI: [-0.164, +0.031] (n=40) — includes 0 ⇒ tie
```

## What we tested

- Same GraphRAG-Bench medical questions (n=40) for both systems
- Same generator/judge stack: `mistral/mistral-small-latest` · embeddings `mistral/mistral-embed`
- Same Mix mode, matched top-k / chunk size, official `generation_eval` Acc
- Fairness: Mix arms always on, RRF fusion, chunk 1200/100, related_chunk=5
- L2 required: official `retrieval_eval` (evidence recall + context relevancy)
- **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE)
- **Why this n:** n=40 (smoke gate — not a release publish claim; use medical-mid n=200)

## Scorecard for decisions

| Layer | Plain meaning | EdgeQuake | LightRAG | Winner |
|-------|---------------|-----------|----------|--------|
| Answer quality (Acc) | Are answers roughly as good? | 0.709 | 0.775 | Tie (CI) |
| Evidence coverage | Did we find the right sources? | 0.934 | 0.964 | LightRAG |
| Context cleanliness | Is the prompt low-noise? | 0.475 | 0.537 | LightRAG |
| Speed (query p50) | Time to answer (ms) | 4582 | 1092 | LightRAG |

- **EQ/LR p50 ratio:** 4.196× (product SLO target ≤ 1.5×)

## By question type (Acc)

| User need | EdgeQuake | LightRAG | Who leads |
|-----------|-----------|----------|-----------|
| Fact lookup | 0.650 | 0.722 | LightRAG |
| Multi-hop reasoning | 0.737 | 0.786 | LightRAG |
| Summarization | 0.834 | 0.867 | LightRAG |
| Creative / open-ended | 0.617 | 0.725 | LightRAG |

## July 2026 landscape

On this fair Acc head-to-head, EdgeQuake is a **LightRAG-class GraphRAG peer** when Acc is statistically tied. In the GraphRAG-Bench literature (ICLR 2026), **HippoRAG2-class** systems define the aspirational **retrieval SOTA** (high evidence recall **and** high context relevancy with compact prompts). Absolute Acc numbers from the academic paper use different models and are **not** directly comparable to these Mistral Acc pins.

## Allowed / forbidden external claims

| Allowed | Forbidden |
|---------|-----------|
| “Peer / statistical tie with LightRAG on Acc under fair pins” (or “point estimate ahead” only if CI excludes 0) | “Beats LightRAG” / “wins Acc” / “#1 GraphRAG-Bench” / “SOTA RAG” without CI excluding 0 and L2 gates |
| “Peer GraphRAG with production stack (Postgres, API, PDF pipeline)” | “#1 on GraphRAG-Bench” without matching paper protocol |
| “Actively closing retrieval noise / multi-hop / latency gaps” | Silent Acc headline = CE+protect without promotion gate |
| “Publish Acc on medical-mid n=200 under fair pins” | Publishing smoke n=40 as the release score |

## How to reproduce

```bash
make bench                 # medical-mid Acc (n=200) + this publish pack
make bench-warm            # query-only (auto latest warm EQ workspace)
make bench001-smoke-acc    # daily smoke gate only (n=40; not release)
```

- Pins: Mix arms on · RRF · chunk 1200/100 · top-k 30 · `mistral/mistral-small-latest` + `mistral/mistral-embed`

## Pointers

- **This publish pack:** `specs/001-benchmark/e2e/artifacts/publish/latest/`
- **Archive:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260723T012653Z`
- **Technical SUMMARY:** same folder / archive `SUMMARY.md`
- **Static business brief:** `specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md`
- **Acc honesty close:** `specs/001-benchmark/001-edgquake-improvements/018-e4-acc-tie-close.md`
