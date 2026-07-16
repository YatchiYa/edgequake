# SPEC-047 smoke — 2026-07-15T13:39:47Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5624** (n_scored=117)
- Overall F1: **0.4575**
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
- page_hit@1: 0.48
- page_hit@3: 0.6933333333333334
- page_hit@5: 0.76
- page_hit@10: 0.84
- page_recall@5: 0.6586666666666666
- mean_n_chunk_sources: 17.28
- mean_arm_local_chunks: 5.986486486486487
- mean_arm_global_chunks: 9.444444444444445
- mean_arm_naive_chunks: 19.253333333333334

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2533 (n=19)
- false_refusal_given_page_hit@5: 0.1579 (n=9 / 57)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8803418803418803
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9829059829059829 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.017094017094017096 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9743589743589743
- arm_global_present_rate: 0.1282051282051282

## Slices
- Single-page Acc: 0.5076
- Cross-page Acc: 0.3333
- Unanswerable Acc: 0.8095

### By evidence source (multi-label / official)
- Chart: Acc=0.3182 (n=22)
- Figure: Acc=0.4762 (n=21)
- Generalized-text (Layout): Acc=0.1818 (n=11)
- Pure-text (Plain-text): Acc=0.3058 (n=26)
- Table: Acc=0.5769 (n=24)

### By evidence source exclusive (len==1)
- Chart: Acc=0.2857 (n=7)
- Figure: Acc=0.5000 (n=16)
- Generalized-text (Layout): Acc=0.2500 (n=4)
- Pure-text (Plain-text): Acc=0.4214 (n=7)
- Table: Acc=0.5897 (n=15)

### Acc attribution (single-run mass)
- list_gold: Acc=0.4331 n=18 score_sum=7.796
- unanswerable: Acc=0.8095 n=42 score_sum=34.000
- other_answerable: Acc=0.4211 n=57 score_sum=24.000

### By document type
- Academic paper: Acc=0.5625 (n=16)
- Administration/Industry file: Acc=0.4444 (n=18)
- Financial report: Acc=0.5762 (n=17)
- Research report / Introduction: Acc=0.5000 (n=54)
- Tutorial/Workshop: Acc=1.0000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc


## Compare
F1: 0.4575 vs 0.4797 (Δ=-0.0222)
Acc: 0.5624 vs 0.5622 (Δ=+0.0002)
page_hit@1: 0.4800 vs 0.4933 (Δ=-0.0133)
page_hit@3: 0.6933 vs 0.6933 (Δ=+0.0000)
page_hit@5: 0.7600 vs 0.7467 (Δ=+0.0133)
page_hit@10: 0.8400 vs 0.8267 (Δ=+0.0133)
Chart exclusive Acc: 0.2857 (n=7) vs 0.2857 (n=7) (Δ=+0.0000)
Chart multi-label Acc: 0.3182 (n=22) vs 0.3182 (n=22) (Δ=+0.0000)

### Paired Acc Δ attribution (a vs compare=baseline)
n_paired=117 improved=9 worsened=9 ΔAcc=+0.0002
  list_gold: +0.0000 Acc points
  unanswerable: +0.0256 Acc points
  other_answerable: -0.0255 Acc points
  note: list_gold mass often = extract normalize (W4), not W1 Chart representation. Do not claim W1 win from Acc alone.

### Fidelity (this): gateable=True n=75 long=0.7021276595744681 Chart_long={'rate': 0.6428571428571429, 'n': 14, 'threshold': 0.5, 'pass': True} Table_long={'rate': 0.6470588235294118, 'n': 17, 'threshold': 0.55, 'pass': True}

### Fidelity (compare): gateable=True n=75 long=0.6595744680851063 Chart_long={'rate': 0.5714285714285714, 'n': 14, 'threshold': 0.5, 'pass': True} Table_long={'rate': 0.5882352941176471, 'n': 17, 'threshold': 0.55, 'pass': True}
