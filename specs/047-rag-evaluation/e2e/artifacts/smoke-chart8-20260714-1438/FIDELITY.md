# SPEC-047 smoke — W1 representation fidelity

- n_answerable_audited: 30
- answer_in_evidence_rate: **0.5000**
- answer_in_document_rate: 0.6000
- representation_miss_n: 15
- retrieval_miss_given_rep_ok_n: 3

## By evidence source
- ?: rate=0.0000 (n=1)
- Chart: rate=0.4000 (n=15)
- Figure: rate=1.0000 (n=3)
- Generalized-text (Layout): rate=0.5714 (n=7)
- Pure-text (Plain-text): rate=0.5500 (n=20)
- Table: rate=0.4545 (n=11)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve).
