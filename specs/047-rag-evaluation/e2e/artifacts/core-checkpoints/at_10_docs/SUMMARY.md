# SPEC-047 core — 2026-07-15T16:14:12Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5285** (n_scored=137)
- Overall F1: **0.4218**
- Docs: 10 | Questions: 137 | Ingest skip: 0
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
- n_answerable_with_diag: 91
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.5054945054945055
- page_hit@3: 0.7032967032967034
- page_hit@5: 0.7692307692307693
- page_hit@10: 0.8021978021978022
- page_recall@5: 0.6823565323565324
- mean_n_chunk_sources: 17.505494505494507
- mean_arm_local_chunks: 7.4
- mean_arm_global_chunks: 8.428571428571429
- mean_arm_naive_chunks: 18.45054945054945

## Refusal diagnostics (020 A2)
- n_answerable: 91
- false_refusal_rate: 0.2857 (n=26)
- false_refusal_given_page_hit@5: 0.2429 (n=17 / 70)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 137
- arms_gated_rate: 0.8686131386861314
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9927007299270073 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0072992700729927005 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9854014598540146
- arm_global_present_rate: 0.1386861313868613

## Slices
- Single-page Acc: 0.4862
- Cross-page Acc: 0.2500
- Unanswerable Acc: 0.8176

### By evidence source (multi-label / official)
- Chart: Acc=0.2727 (n=22)
- Figure: Acc=0.3214 (n=28)
- Generalized-text (Layout): Acc=0.2500 (n=16)
- Pure-text (Plain-text): Acc=0.3532 (n=31)
- Table: Acc=0.4775 (n=29)

### By evidence source exclusive (len==1)
- Chart: Acc=0.2857 (n=7)
- Figure: Acc=0.4000 (n=20)
- Generalized-text (Layout): Acc=0.2000 (n=5)
- Pure-text (Plain-text): Acc=0.6611 (n=9)
- Table: Acc=0.5529 (n=16)

### Acc attribution (single-run mass)
- list_gold: Acc=0.3236 n=21 score_sum=6.796
- unanswerable: Acc=0.8176 n=46 score_sum=37.609
- other_answerable: Acc=0.4000 n=70 score_sum=28.000

### By document type
- Academic paper: Acc=0.5000 (n=16)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5532 (n=17)
- Guidebook: Acc=0.7500 (n=12)
- Research report / Introduction: Acc=0.5000 (n=54)
- Tutorial/Workshop: Acc=0.6667 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
