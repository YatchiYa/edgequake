# EdgeQuake vs LightRAG — Business Performance Report

**Generated:** 2026-07-23T01:24:36Z  
**Task:** GraphRAG-Bench Acc dual-SUT (July 2026 fair pins)  
**Profile:** `LR_INTENT_W_FACT_L2_v1_lrlike_arms_v2`  
**Fixture:** `medical_publish_question_ids_v1` (n=200)  
**Valid run:** `True`

## One-screen first principles

```text
  Task     GraphRAG-Bench/EQ-vs-LR  (same corpus · questions · judge · Mix↔Mix)
  Sample   n=200 medical-mid (50/type) — bootstrap Acc CI is underpowered at smoke n=40; this is the defendable publish ladder before full core
  Acc      EQ 0.718 · LR 0.764 · Δ -0.047
  Δ Acc CI [-0.082, -0.014] (n=200)
  L2       evidence recall EQ 0.940 / LR 0.950 · ctx_rel EQ 0.477 / LR 0.511
  Latency  query p50 EQ 4794 ms / LR 1079 ms
  Verdict  LightRAG ahead on answer quality (CI excludes 0)
```

## Verdict

```text
  LightRAG ahead on answer quality (CI excludes 0)
  Acc   EdgeQuake 0.718  ·  LightRAG 0.764  ·  Δ -0.047
  Δ Acc 95% CI: [-0.082, -0.014] (n=200)
```

## What we tested

- Same GraphRAG-Bench medical questions (n=200) for both systems
- Same generator/judge stack: `mistral/mistral-small-latest` · embeddings `mistral/mistral-embed`
- Same Mix mode, matched top-k / chunk size, official `generation_eval` Acc
- Fairness: Mix arms always on, RRF fusion, chunk 1200/100, related_chunk=5
- L2 required: official `retrieval_eval` (evidence recall + context relevancy)
- **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE)
- **Why this n:** n=200 medical-mid (50/type) — bootstrap Acc CI is underpowered at smoke n=40; this is the defendable publish ladder before full core

## Scorecard for decisions

| Layer | Plain meaning | EdgeQuake | LightRAG | Winner |
|-------|---------------|-----------|----------|--------|
| Answer quality (Acc) | Are answers roughly as good? | 0.718 | 0.764 | LightRAG |
| Evidence coverage | Did we find the right sources? | 0.940 | 0.950 | LightRAG |
| Context cleanliness | Is the prompt low-noise? | 0.477 | 0.511 | LightRAG |
| Speed (query p50) | Time to answer (ms) | 4794 | 1079 | LightRAG |

- **EQ/LR p50 ratio:** 4.443× (product SLO target ≤ 1.5×)

## By question type (Acc)

| User need | EdgeQuake | LightRAG | Who leads |
|-----------|-----------|----------|-----------|
| Fact lookup | 0.710 | 0.735 | LightRAG |
| Multi-hop reasoning | 0.700 | 0.747 | LightRAG |
| Summarization | 0.806 | 0.816 | LightRAG |
| Creative / open-ended | 0.653 | 0.759 | LightRAG |

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
- **Archive:** `specs/001-benchmark/e2e/artifacts/history/medical-mid-20260723T012436Z`
- **Technical SUMMARY:** same folder / archive `SUMMARY.md`
- **Static business brief:** `specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md`
- **Acc honesty close:** `specs/001-benchmark/001-edgquake-improvements/018-e4-acc-tie-close.md`
