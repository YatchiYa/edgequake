# SPEC-047 smoke — 2026-07-11T14:02:21Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.4327** (n_scored=117)
- Overall F1: **0.2617**
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
- page_hit@1: 0.52
- page_hit@3: 0.72
- page_hit@5: 0.8
- page_hit@10: 0.88
- page_recall@5: 0.6953333333333334
- mean_n_chunk_sources: 17.053333333333335
- mean_arm_local_chunks: 9.567567567567568
- mean_arm_global_chunks: 10.875
- mean_arm_naive_chunks: 19.466666666666665

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.1467 (n=11)
- false_refusal_given_page_hit@5: 0.0833 (n=5 / 60)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8888888888888888
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9914529914529915
- arm_global_present_rate: 0.11965811965811966

## Slices
- Single-page Acc: 0.3237
- Cross-page Acc: 0.1944
- Unanswerable Acc: 0.7381

### By evidence source
- Chart: Acc=0.1818 (n=22)
- Figure: Acc=0.2857 (n=21)
- Generalized-text (Layout): Acc=0.2727 (n=11)
- Pure-text (Plain-text): Acc=0.2548 (n=26)
- Table: Acc=0.2760 (n=24)

### By document type
- Academic paper: Acc=0.3125 (n=16)
- Administration/Industry file: Acc=0.5000 (n=18)
- Financial report: Acc=0.2941 (n=17)
- Research report / Introduction: Acc=0.4375 (n=54)
- Tutorial/Workshop: Acc=0.6667 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
