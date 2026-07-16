# SPEC-047 core — 2026-07-15T16:58:59Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5325** (n_scored=190)
- Overall F1: **0.4398**
- Docs: 15 | Questions: 190 | Ingest skip: 0
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
- n_answerable_with_diag: 127
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.5590551181102362
- page_hit@3: 0.7401574803149606
- page_hit@5: 0.7874015748031497
- page_hit@10: 0.84251968503937
- page_recall@5: 0.7225284339457567
- mean_n_chunk_sources: 17.346456692913385
- mean_arm_local_chunks: 9.642857142857142
- mean_arm_global_chunks: 10.176470588235293
- mean_arm_naive_chunks: 18.88976377952756

## Refusal diagnostics (020 A2)
- n_answerable: 127
- false_refusal_rate: 0.2362 (n=30)
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
- Single-page Acc: 0.5082
- Cross-page Acc: 0.3051
- Unanswerable Acc: 0.7716

### By evidence source (multi-label / official)
- Chart: Acc=0.3871 (n=31)
- Figure: Acc=0.3111 (n=45)
- Generalized-text (Layout): Acc=0.2619 (n=22)
- Pure-text (Plain-text): Acc=0.3833 (n=39)
- Table: Acc=0.4696 (n=38)

### By evidence source exclusive (len==1)
- Chart: Acc=0.5000 (n=12)
- Figure: Acc=0.3548 (n=31)
- Generalized-text (Layout): Acc=0.3452 (n=8)
- Pure-text (Plain-text): Acc=0.6950 (n=10)
- Table: Acc=0.5641 (n=21)

### Acc attribution (single-run mass)
- list_gold: Acc=0.2784 n=28 score_sum=7.796
- unanswerable: Acc=0.7716 n=63 score_sum=48.609
- other_answerable: Acc=0.4521 n=99 score_sum=44.762

### By document type
- Academic paper: Acc=0.5000 (n=16)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5532 (n=17)
- Guidebook: Acc=0.6364 (n=22)
- Research report / Introduction: Acc=0.5427 (n=88)
- Tutorial/Workshop: Acc=0.5238 (n=21)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
