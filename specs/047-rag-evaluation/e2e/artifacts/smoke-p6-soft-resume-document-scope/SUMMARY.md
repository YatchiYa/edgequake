# SPEC-047 smoke — 2026-07-11T05:11:41Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.3844** (n_scored=117)
- Overall F1: **0.2235**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=8

## How to read this score
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 75
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.52
- page_hit@3: 0.72
- page_hit@5: 0.76
- page_hit@10: 0.8266666666666667
- page_recall@5: 0.6501481481481481
- mean_n_chunk_sources: 16.466666666666665
- mean_arm_local_chunks: 3.4444444444444446
- mean_arm_global_chunks: 3.8
- mean_arm_naive_chunks: 19.0

## Slices
- Single-page Acc: 0.2559
- Cross-page Acc: 0.1667
- Unanswerable Acc: 0.6905

### By evidence source
- Chart: Acc=0.1364 (n=22)
- Figure: Acc=0.1905 (n=21)
- Generalized-text (Layout): Acc=0.2709 (n=11)
- Pure-text (Plain-text): Acc=0.2692 (n=26)
- Table: Acc=0.1667 (n=24)

### By document type
- Academic paper: Acc=0.1250 (n=16)
- Administration/Industry file: Acc=0.4433 (n=18)
- Financial report: Acc=0.3529 (n=17)
- Research report / Introduction: Acc=0.4259 (n=54)
- Tutorial/Workshop: Acc=0.5000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
