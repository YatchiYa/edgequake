# SPEC-047 smoke — 2026-07-10T09:54:36Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.4068** (n_scored=137)
- Overall F1: **0.2648**
- Docs: 10 | Questions: 137 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_primary` mode=`hybrid`

## How to read this score
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 91
- document_scope: `False`
- context_empty_rate: 0.0000
- page_hit@1: 0.4065934065934066
- page_hit@3: 0.5274725274725275
- page_hit@5: 0.5934065934065934
- page_hit@10: 0.7362637362637363
- page_recall@5: 0.5188034188034187
- mean_n_chunk_sources: 13.505494505494505
- mean_arm_local_chunks: 14.857142857142858
- mean_arm_global_chunks: 14.733333333333333
- mean_arm_naive_chunks: 20.0

## Slices
- Single-page Acc: 0.2300
- Cross-page Acc: 0.2000
- Unanswerable Acc: 0.7826

### By evidence source
- Chart: Acc=0.0455 (n=22)
- Figure: Acc=0.1429 (n=28)
- Generalized-text (Layout): Acc=0.2956 (n=16)
- Pure-text (Plain-text): Acc=0.1935 (n=31)
- Table: Acc=0.3362 (n=29)

### By document type
- Academic paper: Acc=0.3125 (n=16)
- Administration/Industry file: Acc=0.3878 (n=18)
- Brochure: Acc=0.2188 (n=8)
- Financial report: Acc=0.3529 (n=17)
- Guidebook: Acc=0.6667 (n=12)
- Research report / Introduction: Acc=0.4074 (n=54)
- Tutorial/Workshop: Acc=0.5000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
