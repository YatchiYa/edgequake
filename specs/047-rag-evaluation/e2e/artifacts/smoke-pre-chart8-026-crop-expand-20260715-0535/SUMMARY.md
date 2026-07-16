# SPEC-047 smoke — 2026-07-15T05:32:59Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `False` (PARTIAL_INGEST)
- Overall Acc: **0.0000** (n_scored=0)
- Overall F1: **0.0000**
- Docs: 8 | Questions: 0 | Ingest skip: 8
- Ingest coverage: 0.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2 ingest_workers=3

## How to read this score
- **protocol:** `026-hardened-2026-07-15` — Acc/F1 = official soft-score; W1 gates use long-needle a_in_e.
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **by_evidence_source** multi-counts (official). **exclusive** = len(sources)==1 (honest Chart-only).
- **Acc ↑ ≠ W1 win** — require Chart exclusive Acc ↑ and Chart a_in_e_long ≥ 0.50.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- _(no retrieval block — re-run query stage with current bench047)_

## Refusal diagnostics (020 A2)
- _(no refusal block — re-run query stage)_

## Arm-gate diagnostics (020 B1/B2)
- _(no arm-gate block — engine stats omitted)_

## Slices
- Single-page Acc: 0.0000
- Cross-page Acc: 0.0000
- Unanswerable Acc: 0.0000

### By evidence source (multi-label / official)

### By evidence source exclusive (len==1)

### Acc attribution (single-run mass)
- list_gold: Acc=— n=0 score_sum=0.000
- unanswerable: Acc=— n=0 score_sum=0.000
- other_answerable: Acc=— n=0 score_sum=0.000

### By document type

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
