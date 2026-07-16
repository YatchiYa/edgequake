# SPEC-047 smoke — W1 representation fidelity

- n_answerable_audited: 75
- answer_in_evidence_rate: **0.5333**
- answer_in_document_rate: 0.6400
- representation_miss_n: 35
- retrieval_miss_given_rep_ok_n: 13

## By evidence source
- ?: rate=0.0000 (n=2)
- Chart: rate=0.4091 (n=22)
- Figure: rate=0.7143 (n=21)
- Generalized-text (Layout): rate=0.5455 (n=11)
- Pure-text (Plain-text): rate=0.5769 (n=26)
- Table: rate=0.4583 (n=24)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve).
