# SPEC-047 core — 2026-07-16T04:34:02Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4581** (n_scored=397)
- Overall F1: **0.3564**
- Docs: 40 | Questions: 397 | Ingest skip: 0
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
- n_answerable_with_diag: 278
- document_scope: `True`
- context_empty_rate: 0.0683
- page_hit@1: 0.45323741007194246
- page_hit@3: 0.6402877697841727
- page_hit@5: 0.697841726618705
- page_hit@10: 0.7517985611510791
- page_recall@5: 0.6154215089466888
- mean_n_chunk_sources: 15.81294964028777
- mean_arm_local_chunks: 12.856589147286822
- mean_arm_global_chunks: 14.21875
- mean_arm_naive_chunks: 19.166023166023166

## Refusal diagnostics (020 A2)
- n_answerable: 278
- false_refusal_rate: 0.2950 (n=82)
- false_refusal_given_page_hit@5: 0.1701 (n=33 / 194)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 370
- arms_gated_rate: 0.8756756756756757
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9972972972972973 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.002702702702702703 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9945945945945946
- arm_global_present_rate: 0.12702702702702703

## Slices
- Single-page Acc: 0.3986
- Cross-page Acc: 0.2388
- Unanswerable Acc: 0.7782

### By evidence source (multi-label / official)
- Chart: Acc=0.2931 (n=58)
- Figure: Acc=0.2247 (n=112)
- Generalized-text (Layout): Acc=0.1828 (n=37)
- Pure-text (Plain-text): Acc=0.2760 (n=96)
- Table: Acc=0.4396 (n=101)

### By evidence source exclusive (len==1)
- Chart: Acc=0.3200 (n=25)
- Figure: Acc=0.2633 (n=69)
- Generalized-text (Layout): Acc=0.3452 (n=8)
- Pure-text (Plain-text): Acc=0.6318 (n=11)
- Table: Acc=0.4877 (n=53)

### Acc attribution (single-run mass)
- list_gold: Acc=0.2144 n=44 score_sum=9.433
- unanswerable: Acc=0.7782 n=119 score_sum=92.609
- other_answerable: Acc=0.3412 n=234 score_sum=79.840

### By document type
- Academic paper: Acc=0.4034 (n=130)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5156 (n=26)
- Guidebook: Acc=0.4894 (n=47)
- Research report / Introduction: Acc=0.4911 (n=123)
- Tutorial/Workshop: Acc=0.4808 (n=45)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
