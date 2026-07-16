# SPEC-047 core — 2026-07-16T03:06:02Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4665** (n_scored=318)
- Overall F1: **0.3590**
- Docs: 30 | Questions: 318 | Ingest skip: 0
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
- n_answerable_with_diag: 218
- document_scope: `True`
- context_empty_rate: 0.0872
- page_hit@1: 0.45871559633027525
- page_hit@3: 0.6284403669724771
- page_hit@5: 0.6834862385321101
- page_hit@10: 0.7293577981651376
- page_recall@5: 0.6128361954050027
- mean_n_chunk_sources: 15.623853211009175
- mean_arm_local_chunks: 11.929292929292929
- mean_arm_global_chunks: 12.833333333333334
- mean_arm_naive_chunks: 19.050251256281406

## Refusal diagnostics (020 A2)
- n_answerable: 218
- false_refusal_rate: 0.2982 (n=65)
- false_refusal_given_page_hit@5: 0.1678 (n=25 / 149)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 291
- arms_gated_rate: 0.8900343642611683
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9965635738831615 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.003436426116838488 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.993127147766323
- arm_global_present_rate: 0.1134020618556701

## Slices
- Single-page Acc: 0.3991
- Cross-page Acc: 0.2358
- Unanswerable Acc: 0.7761

### By evidence source (multi-label / official)
- Chart: Acc=0.3137 (n=51)
- Figure: Acc=0.2503 (n=86)
- Generalized-text (Layout): Acc=0.2049 (n=33)
- Pure-text (Plain-text): Acc=0.2860 (n=72)
- Table: Acc=0.4234 (n=72)

### By evidence source exclusive (len==1)
- Chart: Acc=0.3500 (n=20)
- Figure: Acc=0.2952 (n=56)
- Generalized-text (Layout): Acc=0.3452 (n=8)
- Pure-text (Plain-text): Acc=0.6950 (n=10)
- Table: Acc=0.4553 (n=37)

### Acc attribution (single-run mass)
- list_gold: Acc=0.2166 n=36 score_sum=7.796
- unanswerable: Acc=0.7761 n=100 score_sum=77.609
- other_answerable: Acc=0.3458 n=182 score_sum=62.931

### By document type
- Academic paper: Acc=0.3712 (n=58)
- Administration/Industry file: Acc=0.5000 (n=18)
- Brochure: Acc=0.2500 (n=8)
- Financial report: Acc=0.5156 (n=26)
- Guidebook: Acc=0.4894 (n=47)
- Research report / Introduction: Acc=0.4911 (n=123)
- Tutorial/Workshop: Acc=0.5000 (n=38)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
