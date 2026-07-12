# SPEC-047 smoke — 2026-07-11T09:24:04Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.3932** (n_scored=117)
- Overall F1: **0.1752**
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
- context_empty_rate: 0.0133
- page_hit@1: 0.4666666666666667
- page_hit@3: 0.68
- page_hit@5: 0.7466666666666667
- page_hit@10: 0.8
- page_recall@5: 0.6368148148148148
- mean_n_chunk_sources: 15.613333333333333
- mean_arm_local_chunks: 0.2465753424657534
- mean_arm_global_chunks: 0.36363636363636365
- mean_arm_naive_chunks: 18.18918918918919

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2800 (n=21)
- false_refusal_given_page_hit@5: 0.1607 (n=9 / 56)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 114
- arms_gated_rate: 0.8771929824561403
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.24561403508771928 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.7543859649122807 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.23684210526315788
- arm_global_present_rate: 0.043859649122807015

## Slices
- Single-page Acc: 0.2051
- Cross-page Acc: 0.1111
- Unanswerable Acc: 0.8095

### By evidence source
- Chart: Acc=0.1818 (n=22)
- Figure: Acc=0.1429 (n=21)
- Generalized-text (Layout): Acc=0.1818 (n=11)
- Pure-text (Plain-text): Acc=0.1923 (n=26)
- Table: Acc=0.0833 (n=24)

### By document type
- Academic paper: Acc=0.2500 (n=16)
- Administration/Industry file: Acc=0.5556 (n=18)
- Financial report: Acc=0.2941 (n=17)
- Research report / Introduction: Acc=0.3704 (n=54)
- Tutorial/Workshop: Acc=0.5833 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
