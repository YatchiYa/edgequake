# SPEC-047 core — 2026-07-15T19:07:48Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5031** (n_scored=217)
- Overall F1: **0.4074**
- Docs: 20 | Questions: 217 | Ingest skip: 2
- Ingest coverage: 0.90
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
- n_answerable_with_diag: 146
- document_scope: `True`
- context_empty_rate: 0.1301
- page_hit@1: 0.4863013698630137
- page_hit@3: 0.6438356164383562
- page_hit@5: 0.684931506849315
- page_hit@10: 0.7328767123287672
- page_recall@5: 0.6285007610350076
- mean_n_chunk_sources: 15.08904109589041
- mean_arm_local_chunks: 9.642857142857142
- mean_arm_global_chunks: 10.176470588235293
- mean_arm_naive_chunks: 18.88976377952756

## Refusal diagnostics (020 A2)
- n_answerable: 146
- false_refusal_rate: 0.3356 (n=49)
- false_refusal_given_page_hit@5: 0.1800 (n=18 / 100)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 190
- arms_gated_rate: 0.8842105263157894
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9947368421052631 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.005263157894736842 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9894736842105263
- arm_global_present_rate: 0.12105263157894737

## Slices
- Single-page Acc: 0.4445
- Cross-page Acc: 0.2687
- Unanswerable Acc: 0.7973

### By evidence source (multi-label / official)
- Chart: Acc=0.3333 (n=36)
- Figure: Acc=0.2727 (n=55)
- Generalized-text (Layout): Acc=0.2401 (n=24)
- Pure-text (Plain-text): Acc=0.3067 (n=52)
- Table: Acc=0.4383 (n=43)

### By evidence source exclusive (len==1)
- Chart: Acc=0.4615 (n=13)
- Figure: Acc=0.3243 (n=37)
- Generalized-text (Layout): Acc=0.3452 (n=8)
- Pure-text (Plain-text): Acc=0.6950 (n=10)
- Table: Acc=0.5385 (n=22)

### Acc attribution (single-run mass)
- list_gold: Acc=0.2436 n=32 score_sum=7.796
- unanswerable: Acc=0.7973 n=71 score_sum=56.609
- other_answerable: Acc=0.3926 n=114 score_sum=44.762

### By document type
- Academic paper: Acc=0.5000 (n=16)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5532 (n=17)
- Guidebook: Acc=0.6364 (n=22)
- Research report / Introduction: Acc=0.4978 (n=106)
- Tutorial/Workshop: Acc=0.4667 (n=30)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
