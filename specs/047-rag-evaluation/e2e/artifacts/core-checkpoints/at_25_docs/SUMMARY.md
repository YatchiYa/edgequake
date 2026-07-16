# SPEC-047 core — 2026-07-16T02:09:48Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4708** (n_scored=268)
- Overall F1: **0.3731**
- Docs: 25 | Questions: 268 | Ingest skip: 1
- Ingest coverage: 0.96
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
- n_answerable_with_diag: 183
- document_scope: `True`
- context_empty_rate: 0.1038
- page_hit@1: 0.46994535519125685
- page_hit@3: 0.644808743169399
- page_hit@5: 0.6939890710382514
- page_hit@10: 0.7431693989071039
- page_recall@5: 0.6407710989678203
- mean_n_chunk_sources: 15.53551912568306
- mean_arm_local_chunks: 11.030674846625766
- mean_arm_global_chunks: 11.210526315789474
- mean_arm_naive_chunks: 18.847560975609756

## Refusal diagnostics (020 A2)
- n_answerable: 183
- false_refusal_rate: 0.3115 (n=57)
- false_refusal_given_page_hit@5: 0.1811 (n=23 / 127)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 241
- arms_gated_rate: 0.8962655601659751
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.995850622406639 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.004149377593360996 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.991701244813278
- arm_global_present_rate: 0.1078838174273859

## Slices
- Single-page Acc: 0.4035
- Cross-page Acc: 0.2593
- Unanswerable Acc: 0.7601

### By evidence source (multi-label / official)
- Chart: Acc=0.3333 (n=42)
- Figure: Acc=0.2429 (n=70)
- Generalized-text (Layout): Acc=0.2504 (n=27)
- Pure-text (Plain-text): Acc=0.3158 (n=60)
- Table: Acc=0.4359 (n=57)

### By evidence source exclusive (len==1)
- Chart: Acc=0.4375 (n=16)
- Figure: Acc=0.2653 (n=49)
- Generalized-text (Layout): Acc=0.3452 (n=8)
- Pure-text (Plain-text): Acc=0.6950 (n=10)
- Table: Acc=0.4499 (n=33)

### Acc attribution (single-run mass)
- list_gold: Acc=0.2293 n=34 score_sum=7.796
- unanswerable: Acc=0.7601 n=85 score_sum=64.609
- other_answerable: Acc=0.3608 n=149 score_sum=53.762

### By document type
- Academic paper: Acc=0.3673 (n=49)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5156 (n=26)
- Guidebook: Acc=0.5484 (n=31)
- Research report / Introduction: Acc=0.4978 (n=106)
- Tutorial/Workshop: Acc=0.4667 (n=30)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
