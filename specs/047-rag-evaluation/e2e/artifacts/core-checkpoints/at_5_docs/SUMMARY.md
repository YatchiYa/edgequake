# SPEC-047 core — 2026-07-15T15:41:24Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5494** (n_scored=79)
- Overall F1: **0.4910**
- Docs: 5 | Questions: 79 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=1 ingest_workers=1

## How to read this score
- **protocol:** `026-listmem-2026-07-15` — Acc/F1 = official soft-score; W1 gates use long-needle a_in_e.
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **by_evidence_source** multi-counts (official). **exclusive** = len(sources)==1 (honest Chart-only).
- **Acc ↑ ≠ W1 win** — require Chart exclusive Acc ↑ and Chart a_in_e_long ≥ 0.50.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 53
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.5471698113207547
- page_hit@3: 0.7358490566037735
- page_hit@5: 0.7547169811320755
- page_hit@10: 0.7924528301886793
- page_recall@5: 0.660062893081761
- mean_n_chunk_sources: 16.69811320754717
- mean_arm_local_chunks: 6.288461538461538
- mean_arm_global_chunks: 6.666666666666667
- mean_arm_naive_chunks: 18.943396226415093

## Refusal diagnostics (020 A2)
- n_answerable: 53
- false_refusal_rate: 0.2453 (n=13)
- false_refusal_given_page_hit@5: 0.2000 (n=8 / 40)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 79
- arms_gated_rate: 0.8354430379746836
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9873417721518988
- arm_global_present_rate: 0.17721518987341772

## Slices
- Single-page Acc: 0.5932
- Cross-page Acc: 0.3043
- Unanswerable Acc: 0.7157

### By evidence source (multi-label / official)
- Chart: Acc=0.4286 (n=7)
- Figure: Acc=0.4000 (n=20)
- Generalized-text (Layout): Acc=0.0000 (n=5)
- Pure-text (Plain-text): Acc=0.5500 (n=9)
- Table: Acc=0.6154 (n=16)

### By evidence source exclusive (len==1)
- Chart: Acc=0.3333 (n=6)
- Figure: Acc=0.4375 (n=16)
- Generalized-text (Layout): Acc=0.0000 (n=4)
- Pure-text (Plain-text): Acc=0.5643 (n=7)
- Table: Acc=0.6319 (n=14)

### Acc attribution (single-run mass)
- list_gold: Acc=0.4854 n=14 score_sum=6.796
- unanswerable: Acc=0.7157 n=26 score_sum=18.609
- other_answerable: Acc=0.4615 n=39 score_sum=18.000

### By document type
- Academic paper: Acc=0.5000 (n=16)
- Administration/Industry file: Acc=0.5000 (n=18)
- Financial report: Acc=0.5532 (n=17)
- Research report / Introduction: Acc=0.5625 (n=16)
- Tutorial/Workshop: Acc=0.6667 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
