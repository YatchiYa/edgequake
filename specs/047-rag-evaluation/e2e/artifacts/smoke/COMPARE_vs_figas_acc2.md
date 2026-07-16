# SPEC-047 smoke — 2026-07-15T13:25:05Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5453** (n_scored=117)
- Overall F1: **0.4287**
- Docs: 8 | Questions: 117 | Ingest skip: 0
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
- n_answerable_with_diag: 75
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.4666666666666667
- page_hit@3: 0.6666666666666666
- page_hit@5: 0.7466666666666667
- page_hit@10: 0.8266666666666667
- page_recall@5: 0.6453333333333333
- mean_n_chunk_sources: 17.266666666666666
- mean_arm_local_chunks: 6.04054054054054
- mean_arm_global_chunks: 10.25
- mean_arm_naive_chunks: 19.253333333333334

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2533 (n=19)
- false_refusal_given_page_hit@5: 0.1250 (n=7 / 56)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 116
- arms_gated_rate: 0.896551724137931
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9913793103448276 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.008620689655172414 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9827586206896551
- arm_global_present_rate: 0.11206896551724138

## Slices
- Single-page Acc: 0.5076
- Cross-page Acc: 0.2778
- Unanswerable Acc: 0.8095

### By evidence source (multi-label / official)
- Chart: Acc=0.2273 (n=22)
- Figure: Acc=0.3741 (n=21)
- Generalized-text (Layout): Acc=0.2718 (n=11)
- Pure-text (Plain-text): Acc=0.3058 (n=26)
- Table: Acc=0.5000 (n=24)

### By evidence source exclusive (len==1)
- Chart: Acc=0.2857 (n=7)
- Figure: Acc=0.3661 (n=16)
- Generalized-text (Layout): Acc=0.4974 (n=4)
- Pure-text (Plain-text): Acc=0.5643 (n=7)
- Table: Acc=0.6000 (n=15)

### Acc attribution (single-run mass)
- list_gold: Acc=0.3306 n=18 score_sum=5.950
- unanswerable: Acc=0.8095 n=42 score_sum=34.000
- other_answerable: Acc=0.4184 n=57 score_sum=23.847

### By document type
- Academic paper: Acc=0.4375 (n=16)
- Administration/Industry file: Acc=0.5550 (n=18)
- Financial report: Acc=0.5265 (n=17)
- Research report / Introduction: Acc=0.4815 (n=54)
- Tutorial/Workshop: Acc=0.9881 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc


## Compare
F1: 0.4287 vs 0.4797 (Δ=-0.0509)
Acc: 0.5453 vs 0.5622 (Δ=-0.0169)
page_hit@1: 0.4667 vs 0.4933 (Δ=-0.0267)
page_hit@3: 0.6667 vs 0.6933 (Δ=-0.0267)
page_hit@5: 0.7467 vs 0.7467 (Δ=+0.0000)
page_hit@10: 0.8267 vs 0.8267 (Δ=+0.0000)
Chart exclusive Acc: 0.2857 (n=7) vs 0.2857 (n=7) (Δ=+0.0000)
Chart multi-label Acc: 0.2273 (n=22) vs 0.3182 (n=22) (Δ=-0.0909)

### Paired Acc Δ attribution (a vs compare=baseline)
n_paired=117 improved=8 worsened=10 ΔAcc=-0.0169
  list_gold: -0.0158 Acc points
  unanswerable: +0.0256 Acc points
  other_answerable: -0.0268 Acc points
  note: list_gold mass often = extract normalize (W4), not W1 Chart representation. Do not claim W1 win from Acc alone.

### Fidelity (this): gateable=True n=75 long=0.7021276595744681 Chart_long={'rate': 0.6428571428571429, 'n': 14, 'threshold': 0.5, 'pass': True} Table_long={'rate': 0.6470588235294118, 'n': 17, 'threshold': 0.55, 'pass': True}

### Fidelity (compare): gateable=True n=75 long=0.6595744680851063 Chart_long={'rate': 0.5714285714285714, 'n': 14, 'threshold': 0.5, 'pass': True} Table_long={'rate': 0.5882352941176471, 'n': 17, 'threshold': 0.55, 'pass': True}
