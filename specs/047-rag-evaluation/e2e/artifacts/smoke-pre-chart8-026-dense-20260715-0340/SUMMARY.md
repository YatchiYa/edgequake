# SPEC-047 smoke — 2026-07-15T03:16:26Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4430** (n_scored=117)
- Overall F1: **0.2616**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite_vision_medium` mode=`hybrid` process_options=`ite` query_workers=2 ingest_workers=4

## How to read this score
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 75
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.41333333333333333
- page_hit@3: 0.6533333333333333
- page_hit@5: 0.7733333333333333
- page_hit@10: 0.8533333333333334
- page_recall@5: 0.6817037037037037
- mean_n_chunk_sources: 16.69333333333333
- mean_arm_local_chunks: 7.013513513513513
- mean_arm_global_chunks: 9.545454545454545
- mean_arm_naive_chunks: 18.16

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2000 (n=15)
- false_refusal_given_page_hit@5: 0.1034 (n=6 / 58)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8547008547008547
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9914529914529915
- arm_global_present_rate: 0.15384615384615385

## Slices
- Single-page Acc: 0.3548
- Cross-page Acc: 0.1389
- Unanswerable Acc: 0.7857

### By evidence source
- Chart: Acc=0.1818 (n=22)
- Figure: Acc=0.2313 (n=21)
- Generalized-text (Layout): Acc=0.2709 (n=11)
- Pure-text (Plain-text): Acc=0.2308 (n=26)
- Table: Acc=0.2500 (n=24)

### By document type
- Academic paper: Acc=0.3750 (n=16)
- Administration/Industry file: Acc=0.4989 (n=18)
- Financial report: Acc=0.4706 (n=17)
- Research report / Introduction: Acc=0.3889 (n=54)
- Tutorial/Workshop: Acc=0.6548 (n=12)


## Stronger vision ablation (025)

- Profile: `P0_mm_ite_vision_medium`
- Query LLM: `mistral-small-latest` (unchanged)
- Vision Pass A/B: `mistral-medium-3-5` (only causal change)
- Gate before Acc claims: Chart `answer_in_evidence` ≥ 0.50 via `bench047 fidelity`
- Baseline: locked `P0_mm_ite` Small+Small chart-8 Acc ~0.415 / Chart a_in_e ~0.40


## vs Small-vision baseline (chart-8 ite SOTA 2026-07-15)

| Metric | Small vision (`P0_mm_ite`) | Medium vision (`P0_mm_ite_vision_medium`) | Δ |
|--------|---------------------------:|------------------------------------------:|--:|
| Acc | 0.4154 | **0.4430** | +0.028 |
| F1 | 0.2464 | 0.2616 | +0.015 |
| Chart Acc | **0.227** | 0.182 | −0.045 |
| Table Acc | 0.193 | **0.250** | +0.057 |
| page_hit@5 | 0.773 | 0.773 | 0 |
| Chart a_in_e (fidelity n≈15) | 0.40 | **0.40** | 0 |

**Go/no-go (025):** Chart `a_in_e` gate **FAIL** (need ≥0.50). Stronger vision alone did **not** move W1 representation for charts. Table Acc improved; Chart Acc did not. Acc +0.028 is within/near noise + Table lift — do **not** claim W1 solved. Next: denser Pass A / crop residual (024 §6).

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
