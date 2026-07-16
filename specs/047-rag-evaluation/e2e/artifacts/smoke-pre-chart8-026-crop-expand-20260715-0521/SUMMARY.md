# SPEC-047 smoke — 2026-07-15T04:14:03Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only.

## Verdict
- valid: `True`
- Overall Acc: **0.4998** (n_scored=117)
- Overall F1: **0.3742**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2 ingest_workers=3

## How to read this score
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 75
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.49333333333333335
- page_hit@3: 0.6533333333333333
- page_hit@5: 0.7466666666666667
- page_hit@10: 0.84
- page_recall@5: 0.662
- mean_n_chunk_sources: 17.293333333333333
- mean_arm_local_chunks: 6.54054054054054
- mean_arm_global_chunks: 9.0
- mean_arm_naive_chunks: 18.96

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2267 (n=17)
- false_refusal_given_page_hit@5: 0.1250 (n=7 / 56)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8632478632478633
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9914529914529915
- arm_global_present_rate: 0.1452991452991453

## Slices
- Single-page Acc: 0.4600
- Cross-page Acc: 0.2500
- Unanswerable Acc: 0.7509

### By evidence source
- Chart: Acc=0.1818 (n=22)
- Figure: Acc=0.4286 (n=21)
- Generalized-text (Layout): Acc=0.3627 (n=11)
- Pure-text (Plain-text): Acc=0.3442 (n=26)
- Table: Acc=0.2917 (n=24)

### By document type
- Academic paper: Acc=0.4375 (n=16)
- Administration/Industry file: Acc=0.4439 (n=18)
- Financial report: Acc=0.4405 (n=17)
- Research report / Introduction: Acc=0.5000 (n=54)
- Tutorial/Workshop: Acc=0.7500 (n=12)


## vs LVLM SOTA (July 2026 reference) — READ CAVEATS

**Task identity:** this EdgeQuake run is a **RAG adaptation** on the chart-8 smoke fixture
(8 docs / 117 Qs, hybrid retrieve + Small LLM).
Official MMLongBench-Doc leaderboard scores are **page-screenshot LVLMs on ~1082 questions**.
Numbers are **difficulty references**, not a same-protocol ranking.

| System | Acc | F1 | Chart Acc | Protocol |
|--------|-----|----|-----------|----------|
| **EdgeQuake P0_mm_ite (this run)** | **0.4998** | **0.3742** | **0.1818** | RAG · 8-doc smoke · dscope · ite |
| TeleMM2.0 (2026-01-05) — official HF SOTA | 0.5609 | 0.5590 | 0.5416 | Full LVLM board |
| GPT-4.1 (2025-04-14) | 0.4974 | 0.5142 | 0.4847 | Full LVLM board |
| GPT-4o (2024-11-20, refreshed board) | 0.4625 | 0.4624 | 0.4315 | Full LVLM board |
| Paper GPT-4o (NeurIPS'24 original report) | — | 0.4490 | — | Full LVLM board |

Sources: [OpenIXCLab/mmlongbench-doc-results](https://huggingface.co/datasets/OpenIXCLab/mmlongbench-doc-results)
(official). Aggregators may list higher single scores (e.g. Qwen / Nemotron ~57–62%) under
third-party protocols — prefer the official Acc/F1 board for citation.

- ΔAcc vs TeleMM2.0 (SOTA Acc): **-0.0611** (not same task)
- ΔF1 vs TeleMM2.0 (SOTA F1): **-0.1848** (not same task)
- ΔF1 vs paper GPT-4o (0.449): **-0.0748** (difficulty ref only)
- Ops: ingest_coverage=1.0 page_hit@5=0.7466666666666667 empty=0.0
## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
