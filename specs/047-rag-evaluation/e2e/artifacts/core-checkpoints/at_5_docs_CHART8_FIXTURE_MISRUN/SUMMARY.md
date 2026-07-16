# SPEC-047 core — 2026-07-15T14:47:10Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4847** (n_scored=75)
- Overall F1: **0.3472**
- Docs: 5 | Questions: 75 | Ingest skip: 0
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
- n_answerable_with_diag: 47
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.46808510638297873
- page_hit@3: 0.6808510638297872
- page_hit@5: 0.7021276595744681
- page_hit@10: 0.7872340425531915
- page_recall@5: 0.5677304964539007
- mean_n_chunk_sources: 16.70212765957447
- mean_arm_local_chunks: 5.304347826086956
- mean_arm_global_chunks: 5.5
- mean_arm_naive_chunks: 18.893617021276597

## Refusal diagnostics (020 A2)
- n_answerable: 47
- false_refusal_rate: 0.2340 (n=11)
- false_refusal_given_page_hit@5: 0.1818 (n=6 / 33)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 75
- arms_gated_rate: 0.88
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9866666666666667
- arm_global_present_rate: 0.13333333333333333

## Slices
- Single-page Acc: 0.5118
- Cross-page Acc: 0.1364
- Unanswerable Acc: 0.7343

### By evidence source (multi-label / official)
- Chart: Acc=0.2143 (n=14)
- Figure: Acc=0.1250 (n=8)
- Generalized-text (Layout): Acc=0.0000 (n=5)
- Pure-text (Plain-text): Acc=0.3300 (n=15)
- Table: Acc=0.4904 (n=16)

### By evidence source exclusive (len==1)
- Chart: Acc=0.3333 (n=6)
- Figure: Acc=0.1429 (n=7)
- Generalized-text (Layout): Acc=0.0000 (n=4)
- Pure-text (Plain-text): Acc=0.6583 (n=6)
- Table: Acc=0.6224 (n=11)

### Acc attribution (single-run mass)
- list_gold: Acc=0.3689 n=13 score_sum=4.796
- unanswerable: Acc=0.7343 n=28 score_sum=20.560
- other_answerable: Acc=0.3235 n=34 score_sum=11.000

### By document type
- Academic paper: Acc=0.5000 (n=16)
- Administration/Industry file: Acc=0.3889 (n=18)
- Financial report: Acc=0.6092 (n=17)
- Research report / Introduction: Acc=0.4583 (n=24)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
