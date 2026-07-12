# SPEC-047 smoke — W1 representation fidelity

- n_answerable_audited: 91
- answer_in_evidence_rate: **0.5055**
- answer_in_document_rate: 0.6484
- representation_miss_n: 45
- retrieval_miss_given_rep_ok_n: 16

## By evidence source
- ?: rate=0.3333 (n=3)
- Chart: rate=0.3636 (n=22)
- Figure: rate=0.5714 (n=28)
- Generalized-text (Layout): rate=0.5625 (n=16)
- Pure-text (Plain-text): rate=0.5484 (n=31)
- Table: rate=0.4483 (n=29)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve).
