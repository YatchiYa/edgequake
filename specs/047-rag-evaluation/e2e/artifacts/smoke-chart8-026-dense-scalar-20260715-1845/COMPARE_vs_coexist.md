# SPEC-047 smoke — 2026-07-15T11:58:40Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.5452** (n_scored=117)
- Overall F1: **0.4080**
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
- page_hit@1: 0.4533333333333333
- page_hit@3: 0.7066666666666667
- page_hit@5: 0.76
- page_hit@10: 0.8666666666666667
- page_recall@5: 0.6523703703703704
- mean_n_chunk_sources: 17.08
- mean_arm_local_chunks: 6.121621621621622
- mean_arm_global_chunks: 8.777777777777779
- mean_arm_naive_chunks: 19.14666666666667

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.1600 (n=12)
- false_refusal_given_page_hit@5: 0.0877 (n=5 / 57)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8888888888888888
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9914529914529915 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.008547008547008548 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9829059829059829
- arm_global_present_rate: 0.11965811965811966

## Slices
- Single-page Acc: 0.4817
- Cross-page Acc: 0.3056
- Unanswerable Acc: 0.8095

### By evidence source (multi-label / official)
- Chart: Acc=0.1818 (n=22)
- Figure: Acc=0.5238 (n=21)
- Generalized-text (Layout): Acc=0.1809 (n=11)
- Pure-text (Plain-text): Acc=0.1904 (n=26)
- Table: Acc=0.4519 (n=24)

### By evidence source exclusive (len==1)
- Chart: Acc=0.2857 (n=7)
- Figure: Acc=0.5625 (n=16)
- Generalized-text (Layout): Acc=0.2474 (n=4)
- Pure-text (Plain-text): Acc=0.4214 (n=7)
- Table: Acc=0.5897 (n=15)

### Acc attribution (single-run mass)
- list_gold: Acc=0.3776 n=18 score_sum=6.796
- unanswerable: Acc=0.8095 n=42 score_sum=34.000
- other_answerable: Acc=0.4033 n=57 score_sum=22.990

### By document type
- Academic paper: Acc=0.6250 (n=16)
- Administration/Industry file: Acc=0.4994 (n=18)
- Financial report: Acc=0.5762 (n=17)
- Research report / Introduction: Acc=0.4259 (n=54)
- Tutorial/Workshop: Acc=1.0000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc


## Compare
F1: 0.4080 vs 0.4572 (Δ=-0.0492)
Acc: 0.5452 vs 0.5634 (Δ=-0.0182)
page_hit@1: 0.4533 vs 0.4933 (Δ=-0.0400)
page_hit@3: 0.7067 vs 0.6800 (Δ=+0.0267)
page_hit@5: 0.7600 vs 0.7733 (Δ=-0.0133)
page_hit@10: 0.8667 vs 0.8800 (Δ=-0.0133)
Chart exclusive Acc: 0.2857 (n=7) vs 0.1429 (n=7) (Δ=+0.1429)
Chart multi-label Acc: 0.1818 (n=22) vs 0.3182 (n=22) (Δ=-0.1364)

### Paired Acc Δ attribution (a vs compare=baseline)
n_paired=117 improved=13 worsened=14 ΔAcc=-0.0182
  list_gold: -0.0171 Acc points
  unanswerable: +0.0085 Acc points
  other_answerable: -0.0097 Acc points
  note: list_gold mass often = extract normalize (W4), not W1 Chart representation. Do not claim W1 win from Acc alone.

### Fidelity (this): gateable=True n=75 long=0.7021276595744681 Chart_long={'rate': 0.5714285714285714, 'n': 14, 'threshold': 0.5, 'pass': True} Table_long={'rate': 0.6470588235294118, 'n': 17, 'threshold': 0.55, 'pass': True}

### Fidelity (compare): gateable=True n=75 long=0.3829787234042553 Chart_long={'rate': 0.21428571428571427, 'n': 14, 'threshold': 0.5, 'pass': False} Table_long={'rate': 0.35294117647058826, 'n': 17, 'threshold': 0.55, 'pass': False}
