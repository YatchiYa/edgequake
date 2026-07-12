# SPEC-047 smoke — 2026-07-11T10:24:57Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.4272** (n_scored=117)
- Overall F1: **0.2253**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2

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
- page_hit@1: 0.48
- page_hit@3: 0.6933333333333334
- page_hit@5: 0.76
- page_hit@10: 0.8133333333333334
- page_recall@5: 0.6434814814814815
- mean_n_chunk_sources: 15.84
- mean_arm_local_chunks: 0.24324324324324326
- mean_arm_global_chunks: 0.2727272727272727
- mean_arm_naive_chunks: 18.213333333333335

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.3200 (n=24)
- false_refusal_given_page_hit@5: 0.2105 (n=12 / 57)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8717948717948718
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.23931623931623933 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.7606837606837606 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.23076923076923078
- arm_global_present_rate: 0.03418803418803419

## Slices
- Single-page Acc: 0.2815
- Cross-page Acc: 0.1111
- Unanswerable Acc: 0.8333

### By evidence source
- Chart: Acc=0.1364 (n=22)
- Figure: Acc=0.2381 (n=21)
- Generalized-text (Layout): Acc=0.1800 (n=11)
- Pure-text (Plain-text): Acc=0.1923 (n=26)
- Table: Acc=0.1250 (n=24)

### By document type
- Academic paper: Acc=0.3125 (n=16)
- Administration/Industry file: Acc=0.4433 (n=18)
- Financial report: Acc=0.3529 (n=17)
- Research report / Introduction: Acc=0.4630 (n=54)
- Tutorial/Workshop: Acc=0.5000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
