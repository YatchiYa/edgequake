# SPEC-047 core — 2026-07-16T03:52:41Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4588** (n_scored=358)
- Overall F1: **0.3563**
- Docs: 35 | Questions: 358 | Ingest skip: 0
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
- n_answerable_with_diag: 248
- document_scope: `True`
- context_empty_rate: 0.0766
- page_hit@1: 0.4475806451612903
- page_hit@3: 0.625
- page_hit@5: 0.6774193548387096
- page_hit@10: 0.7258064516129032
- page_recall@5: 0.6106113868210642
- mean_n_chunk_sources: 15.806451612903226
- mean_arm_local_chunks: 12.18421052631579
- mean_arm_global_chunks: 13.346153846153847
- mean_arm_naive_chunks: 19.100436681222707

## Refusal diagnostics (020 A2)
- n_answerable: 248
- false_refusal_rate: 0.3024 (n=75)
- false_refusal_given_page_hit@5: 0.1667 (n=28 / 168)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 331
- arms_gated_rate: 0.8942598187311178
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9969788519637462 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0030211480362537764 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9939577039274925
- arm_global_present_rate: 0.10876132930513595

## Slices
- Single-page Acc: 0.4014
- Cross-page Acc: 0.2289
- Unanswerable Acc: 0.7692

### By evidence source (multi-label / official)
- Chart: Acc=0.3091 (n=55)
- Figure: Acc=0.2401 (n=98)
- Generalized-text (Layout): Acc=0.1878 (n=36)
- Pure-text (Plain-text): Acc=0.2849 (n=86)
- Table: Acc=0.4136 (n=88)

### By evidence source exclusive (len==1)
- Chart: Acc=0.3478 (n=23)
- Figure: Acc=0.2755 (n=60)
- Generalized-text (Layout): Acc=0.3452 (n=8)
- Pure-text (Plain-text): Acc=0.6950 (n=10)
- Table: Acc=0.4410 (n=45)

### Acc attribution (single-run mass)
- list_gold: Acc=0.2315 n=38 score_sum=8.796
- unanswerable: Acc=0.7692 n=110 score_sum=84.609
- other_answerable: Acc=0.3373 n=210 score_sum=70.840

### By document type
- Academic paper: Acc=0.3820 (n=98)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5156 (n=26)
- Guidebook: Acc=0.4894 (n=47)
- Research report / Introduction: Acc=0.4911 (n=123)
- Tutorial/Workshop: Acc=0.5000 (n=38)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
