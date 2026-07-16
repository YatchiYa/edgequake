# SPEC-047 smoke — W1 representation fidelity

- n_answerable_audited: 25
- answer_in_evidence_rate: **0.4800**
- answer_in_document_rate: 0.6000
- representation_miss_n: 13
- retrieval_miss_given_rep_ok_n: 2

## By evidence source
- ?: rate=0.0000 (n=1)
- Chart: rate=0.4000 (n=15)
- Figure: rate=1.0000 (n=2)
- Generalized-text (Layout): rate=0.4000 (n=5)
- Pure-text (Plain-text): rate=0.5333 (n=15)
- Table: rate=0.4545 (n=11)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve).
