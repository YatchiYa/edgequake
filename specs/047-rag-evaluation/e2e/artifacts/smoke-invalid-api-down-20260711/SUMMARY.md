# SPEC-047 smoke — 2026-07-11T01:36:57Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `False` (PARTIAL_INGEST)
- Overall Acc: **0.5000** (n_scored=12)
- Overall F1: **0.0000**
- Docs: 8 | Questions: 12 | Ingest skip: 7
- Ingest coverage: 0.12
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=30

## How to read this score
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 6
- document_scope: `False`
- context_empty_rate: 1.0000
- page_hit@1: 0.0
- page_hit@3: 0.0
- page_hit@5: 0.0
- page_hit@10: 0.0
- page_recall@5: 0.0
- mean_n_chunk_sources: 0.0
- mean_arm_local_chunks: None
- mean_arm_global_chunks: None
- mean_arm_naive_chunks: None

## Slices
- Single-page Acc: 0.0000
- Cross-page Acc: 0.0000
- Unanswerable Acc: 1.0000

### By evidence source
- Chart: Acc=0.0000 (n=6)
- Pure-text (Plain-text): Acc=0.0000 (n=4)
- Table: Acc=0.0000 (n=1)

### By document type
- Research report / Introduction: Acc=0.5000 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
