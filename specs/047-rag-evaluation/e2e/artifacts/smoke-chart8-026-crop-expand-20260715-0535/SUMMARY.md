# SPEC-047 smoke — 2026-07-15T06:51:14Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5060** (n_scored=117)
- Overall F1: **0.4028**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=1 ingest_workers=1

## How to read this score
- **protocol:** `026-hardened-2026-07-15` — Acc/F1 = official soft-score; W1 gates use long-needle a_in_e.
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **by_evidence_source** multi-counts (official). **exclusive** = len(sources)==1 (honest Chart-only).
- **Acc ↑ ≠ W1 win** — require Chart exclusive Acc ↑ and Chart a_in_e_long ≥ 0.50.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 75
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.49333333333333335
- page_hit@3: 0.6933333333333334
- page_hit@5: 0.7466666666666667
- page_hit@10: 0.8666666666666667
- page_recall@5: 0.6538518518518519
- mean_n_chunk_sources: 17.52
- mean_arm_local_chunks: 5.648648648648648
- mean_arm_global_chunks: 7.7
- mean_arm_naive_chunks: 19.12

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2267 (n=17)
- false_refusal_given_page_hit@5: 0.1607 (n=9 / 56)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8717948717948718
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9914529914529915
- arm_global_present_rate: 0.13675213675213677

## Slices
- Single-page Acc: 0.4925
- Cross-page Acc: 0.2778
- Unanswerable Acc: 0.7143

### By evidence source (multi-label / official)
- Chart: Acc=0.2727 (n=22)
- Figure: Acc=0.4571 (n=21)
- Generalized-text (Layout): Acc=0.3636 (n=11)
- Pure-text (Plain-text): Acc=0.3058 (n=26)
- Table: Acc=0.4440 (n=24)

### By evidence source exclusive (len==1)
- Chart: Acc=0.2857 (n=7)
- Figure: Acc=0.4125 (n=16)
- Generalized-text (Layout): Acc=0.2500 (n=4)
- Pure-text (Plain-text): Acc=0.4214 (n=7)
- Table: Acc=0.5770 (n=15)

### Acc attribution (single-run mass)
- list_gold: Acc=0.3220 n=18 score_sum=5.796
- unanswerable: Acc=0.7143 n=42 score_sum=30.000
- other_answerable: Acc=0.4107 n=57 score_sum=23.410

### By document type
- Academic paper: Acc=0.4375 (n=16)
- Administration/Industry file: Acc=0.5000 (n=18)
- Financial report: Acc=0.5062 (n=17)
- Research report / Introduction: Acc=0.4630 (n=54)
- Tutorial/Workshop: Acc=0.8000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
