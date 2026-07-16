# SPEC-047 smoke — 2026-07-15T05:24:12Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `False` (PARTIAL_INGEST)
- Overall Acc: **0.0000** (n_scored=0)
- Overall F1: **0.0000**
- Docs: 8 | Questions: 0 | Ingest skip: 8
- Ingest coverage: 0.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2 ingest_workers=4

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


## vs LVLM SOTA (July 2026 reference) — READ CAVEATS

**Task identity:** this EdgeQuake run is a **RAG adaptation** on the chart-8 smoke fixture
(8 docs / 0 Qs, hybrid retrieve + Small LLM).
Official MMLongBench-Doc leaderboard scores are **page-screenshot LVLMs on ~1082 questions**.
Numbers are **difficulty references**, not a same-protocol ranking.

| System | Acc | F1 | Chart Acc | Protocol |
|--------|-----|----|-----------|----------|
| **EdgeQuake P0_mm_ite (this run)** | **0.0000** | **0.0000** | **nan** | RAG · 8-doc smoke · dscope · ite |
| TeleMM2.0 (2026-01-05) — official HF SOTA | 0.5609 | 0.5590 | 0.5416 | Full LVLM board |
| GPT-4.1 (2025-04-14) | 0.4974 | 0.5142 | 0.4847 | Full LVLM board |
| GPT-4o (2024-11-20, refreshed board) | 0.4625 | 0.4624 | 0.4315 | Full LVLM board |
| Paper GPT-4o (NeurIPS'24 original report) | — | 0.4490 | — | Full LVLM board |

Sources: [OpenIXCLab/mmlongbench-doc-results](https://huggingface.co/datasets/OpenIXCLab/mmlongbench-doc-results)
(official). Aggregators may list higher single scores (e.g. Qwen / Nemotron ~57–62%) under
third-party protocols — prefer the official Acc/F1 board for citation.

- ΔAcc vs TeleMM2.0 (SOTA Acc): **-0.5609** (not same task)
- ΔF1 vs TeleMM2.0 (SOTA F1): **-0.5590** (not same task)
- ΔF1 vs paper GPT-4o (0.449): **-0.4490** (difficulty ref only)
- Ops: ingest_coverage=0.0 page_hit@5=None empty=0.0
## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
