# SPEC-047 smoke — W1 representation fidelity

- n_answerable_audited: 6
- answer_in_evidence_rate: **0.5000**
- answer_in_document_rate: 0.5000
- representation_miss_n: 3
- retrieval_miss_given_rep_ok_n: 0

## By evidence source
- Chart: rate=0.5000 (n=6)
- Pure-text (Plain-text): rate=0.7500 (n=4)
- Table: rate=0.0000 (n=1)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve).
