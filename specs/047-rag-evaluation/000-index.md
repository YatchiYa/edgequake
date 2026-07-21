# SPEC-047 — EdgeQuake RAG Evaluation on MMLongBench-Doc

**Status:** HARNESS LIVE · chart-8 Acc **~0.42–0.44** · Medium vision ablation **failed** Chart a_in_e gate (still 0.40) · next: [026](./026-first-principles-score-improvement-brainstorm.md) Wave 1  
**Re-assessment:** [022](./022-reassessment-2026-07-11.md) · Medium result [025](./025-stronger-vision-first-battle-plan.md) · **next FP [026](./026-first-principles-score-improvement-brainstorm.md)**  
**Primary benchmark:** [MMLongBench-Doc](https://github.com/mayubo2333/MMLongBench-Doc) (NeurIPS 2024 D&B)  
**Provider stack (locked for v1):** Mistral Small (LLM + vision) + `mistral-embed` · Postgres  
**Query mode (locked for v1):** `hybrid`  
**Law:** Real PDFs · official Q&A · official scoring · reproducible artifacts · no leaderboard cosplay

---

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  MMLongBench-Doc is an LVLM long-document benchmark (page images in-context).│
│  EdgeQuake is a GraphRAG system (ingest → retrieve → generate).              │
│                                                                              │
│  Chart-8 Acc ~0.42 (Small) / ~0.44 (Medium vision) — Chart a_in_e still 0.40 │
│  Medium ablation FAILED W1 gate → capacity ≠ fidelity (see 025 / 026).       │
│  Dominant fail: zero+hit+wrong (~35); Tables+Pure-text ≥ Chart in that mass. │
│  Next: 026 Wave 1 — denser Pass A, crop telemetry, table specialize.         │
│                                                                              │
│  Progression:  smoke (chart-8)  →  core (≈40)  →  full (135 / 1091 Qs)       │
└──────────────────────────────────────────────────────────────────────────────┘
```

See [022 re-assessment](./022-reassessment-2026-07-11.md), [012 — how to read Acc/F1](./012-acceptance-criteria-and-scorecard.md), and
[`e2e/artifacts/smoke-post-mv18-full-chart/SUMMARY.md`](./e2e/artifacts/smoke-post-mv18-full-chart/SUMMARY.md).

---

## Document map

| # | Doc | Lens / purpose |
|---|-----|----------------|
| 000 | [INDEX](./000-index.md) | Hub, progression, quick start |
| 001 | [First Principles](./001-first-principles.md) | Irreducible axioms for fair RAG eval |
| 002 | [Benchmark Deep Dive](./002-benchmark-deep-dive-mmlongbench.md) | What MMLongBench-Doc actually measures |
| 003 | [Fair Evaluation Protocol](./003-fair-evaluation-protocol.md) | Three-stage scoring adapted for RAG |
| 004 | [AI Engineer Lens](./004-ai-engineer-lens.md) | System design, retrieval physics, failure modes |
| 005 | [Expert Developer Lens](./005-expert-developer-lens.md) | Code layout, APIs, harness architecture |
| 006 | [MLOps Lens](./006-mlops-lens.md) | Repro, cost, caching, CI gates, artifacts |
| 007 | [ML Scientist Lens](./007-ml-scientist-lens.md) | Metrics, stratification, significance, ablations |
| 008 | [Product / SRE Lens](./008-product-sre-lens.md) | Operability, SLOs, cost envelopes, risk |
| 009 | [Implementation Plan](./009-implementation-plan.md) | Tickets EQ-047-01…N, ordered delivery |
| 010 | [Smoke → Full Runbook](./010-smoke-then-full-runbook.md) | Copy-paste commands, expected outputs |
| 011 | [Complementary Benchmarks](./011-complementary-benchmarks-methodology.md) | What else to run + methodology |
| 012 | [Acceptance & Scorecard](./012-acceptance-criteria-and-scorecard.md) | Done-when, schema, gates, score reading |
| 013 | [Improvement Roadmap](./013-first-principles-improvement-roadmap.md) | Code-is-law workstreams; W0 `page_hit@k` live |
| 014 | [Ingest/Query Pipeline Study](./014-ingest-query-pipeline-first-principles.md) | End-to-end code law + ranked lifts |
| 015 | [Modality-Aware Vision Plan](./015-modality-aware-vision-improvement-plan.md) | Chart/figure/table typed prompts + phases · **next Acc lever** |
| 016 | [Ingest Speed & Reliability](./016-ingest-speed-reliability-battle-plan.md) | P0–P7f battle plan (unique embed, merge gates) |
| 017 | [LightRAG vs EdgeQuake Assessment](./017-lightrag-vs-edgequake-query-pipeline-assessment.md) | Code-is-law query/ingest compare · naming trap |
| 018 | [Quality & Speed Improvement Plan](./018-quality-speed-improvement-plan.md) | Phased tickets A–E · gates · experiment order |
| 019 | [Query First-Principles Plan](./019-query-first-principles-improvement-plan.md) | Diagnose R/G/Gen/Rep · Q1 grounding landed |
| 020 | [Post-Q1 First-Principles Plan](./020-post-q1-first-principles-improvement-plan.md) | A1–A3 + B1–B2 done · **B3 Mix next** · 015 hand-off |
| 021 | [Lineage First Principles (Query)](./021-lineage-first-principles-query.md) | Entity→Chunk→Doc→Page · **L-A1–A4 done** · L-B* open |
| **022** | **[Re-Assessment 2026-07-11](./022-reassessment-2026-07-11.md)** | **Authoritative Acc chain + next queue** |
| **023** | **[Next from MV-18/19 (FP)](./023-first-principles-next-from-mv18.md)** | Chart Rep next levers · no Acc heuristics |
| 024 | [Code Acc Bottleneck (FP)](./024-first-principles-code-acc-bottleneck.md) | Call graph · ranked code levers · data-adjusted ranks |
| 025 | [Stronger Vision First](./025-stronger-vision-first-battle-plan.md) | Medium ablation · **gate FAIL** (a_in_e flat 0.40) |
| **026** | **[Score Improvement Brainstorm (FP)](./026-first-principles-score-improvement-brainstorm.md)** | **Authoritative next plan** · Acc #4 W3-arith in flight |
| 027 | [Coexist Acc FP Analysis](./027-coexist-acc-first-principles-analysis.md) | Plumbing Acc · pre-listmem Chart gate |
| 028 | [Fig-as-chart Acc #2](./028-fig-as-chart-acc2-assessment.md) | Acc/F1 SOTA reference |
| 029–030 | [Listmem measure](./029-post-acc2-fp-plan-w1-measure.md) · [assess](./030-w1-measure-listmem-assessment.md) | Chart long gate PASS |
| 031 | [Dense-scalar Acc #3](./031-acc3-dense-scalar-assessment.md) | **NEGATIVE** — densify reverted |
| **032** | **[Post Acc #3 FP · W3-arith](./032-post-acc3-fp-derived-counts-w3-arith.md)** | **Acc #4 stack** · derived counts |
| 033 | [Acc #4 mid-assess](./033-acc4-w3-arith-mid-assessment.md) | Year-span Chart long 0.643 |
| 034 | [Acc #4 final](./034-acc4-w3-arith-assessment.md) | Acc/F1 no lift; Chart long 0.643 |
| **035** | **[Acc #5 W3-v2](./035-acc5-w3-arith-v2-assessment.md)** | Acc≈#2 · 1251 hit · F1 short |
| 036–037 | Phase B fixture fix / status | chart-8 misrun VOID |
| **038** | **[Phase B CORE @5](./038-phase-b-core-checkpoint-at5.md)** | Acc **0.549** · F1 **0.491** |
| — | [e2e/](./e2e/) | Artifacts + how to read SUMMARY |
| — | [fixtures/](./fixtures/) | Smoke doc-id list, stratified seeds |

**Cross-refs:** SPEC-046 GraphRAG study · **[SPEC-001 EQ vs LightRAG](../001-benchmark/000-index.md)** · SPEC-021 Mistral storage proof · SPEC-013 Mistral live ingest · `edgequake/models.toml` Mistral provider block

---

## Locked provider profile (v1)

| Role                  | Env                                    | Value                              | Why                                       |
| -----------------------| ----------------------------------------| ------------------------------------| -------------------------------------------|
| LLM (extract + query) | `EDGEQUAKE_LLM_PROVIDER` / model       | `mistral` / `mistral-small-latest` | Cost/quality balance for smoke→full       |
| Vision (PDF pages)    | `EDGEQUAKE_VISION_PROVIDER` / model    | `mistral` / `mistral-small-latest` | Same Small multimodal model (not Pixtral) |
| Embeddings            | `EDGEQUAKE_EMBEDDING_PROVIDER` / model | `mistral` / `mistral-embed`        | Fixed **1024-d**; proven in SPEC-021      |
| Storage               | `DATABASE_URL`                         | PostgreSQL (required)              | No in-memory mode for bench               |
| Query mode            | request body                           | `hybrid`                           | Local ∥ Global ∥ Naive fusion             |
| Answer extractor      | harness                                | Mistral judge (default) or GPT-4o  | Label in scorecard                        |

> **Ops note:** Hybrid/Mix arm futures are `Box::pin`’d (stack-overflow fix). Tokio worker stack defaults to 8 MiB (`TOKIO_WORKER_STACK_SIZE`). Bench API often `:8090` (see `.edgequake-dev-ports.env`).

---

## Progression (see results grow)

```text
Stage A — SMOKE          Stage B — CORE           Stage C — FULL
───────────────          ──────────────           ──────────────
8–10 real PDFs           ~40 PDFs                 135 PDFs
chart fixture locked     stratified Q subset      all 1091 questions
Acc band ~0.43           prove signal             publish scorecard
make bench047-smoke      make bench047-core       make bench047-full
```

Each stage writes the **same** scorecard schema ([012](./012-acceptance-criteria-and-scorecard.md)). Diff stages side-by-side to see progression. Prefer [022](./022-reassessment-2026-07-11.md) when comparing Acc across query tickets.

---

## Quick start

```bash
export MISTRAL_API_KEY=...
export EDGEQUAKE_API_URL=http://127.0.0.1:8090   # locked bench backend

make postgres-start
# start backend: Mistral Small LLM+vision + mistral-embed (see [010](./010-smoke-then-full-runbook.md))

# Query-only against existing workspace (no re-ingest):
cd tools/bench047 && ./.venv/bin/python -m bench047.cli smoke \
  --api http://127.0.0.1:8090 --profile P0_mm_ite \
  --query-only --document-scope --no-resume --workers 2

cat specs/047-rag-evaluation/e2e/artifacts/smoke/SUMMARY.md
```

---

## Non-goals (v1)

- Claiming parity with the official LVLM leaderboard ([HF Space](https://huggingface.co/spaces/OpenIXCLab/mmlongbench-doc))
- Training / fine-tuning models
- Replacing SPEC-046 GraphRAG-Bench ACC work
- Evaluating non-Mistral stacks in the same run (separate profile later)

---

## License & ethics

MMLongBench-Doc data is **CC BY-NC 4.0** (research / non-commercial). Code in the upstream repo is Apache-2.0. EdgeQuake harness must:

1. Download from official HF / GitHub sources only  
2. Keep PDFs out of git (gitignore + local cache dir)  
3. Cite the paper in every published scorecard  

Paper: [arXiv:2407.01523](https://arxiv.org/abs/2407.01523) · Repo: [mayubo2333/MMLongBench-Doc](https://github.com/mayubo2333/MMLongBench-Doc)

---

## Reading order

1. [001 First Principles](./001-first-principles.md) — do not skip  
2. [022 Re-Assessment](./022-reassessment-2026-07-11.md) — **current Acc + next queue**  
3. [002 Deep Dive](./002-benchmark-deep-dive-mmlongbench.md) + [003 Protocol](./003-fair-evaluation-protocol.md)  
4. Lens docs matching your role (004–008)  
5. [009 Plan](./009-implementation-plan.md) + [010 Runbook](./010-smoke-then-full-runbook.md)  
6. [012](./012-acceptance-criteria-and-scorecard.md) as the definition of done  
7. [019](./019-query-first-principles-improvement-plan.md)–[021](./021-lineage-first-principles-query.md) for query/lineage execution  
8. [015](./015-modality-aware-vision-improvement-plan.md) before expecting Chart Acc lifts  
9. [013](./013-first-principles-improvement-roadmap.md)–[016](./016-ingest-speed-reliability-battle-plan.md) for deeper causal / ingest detail  
