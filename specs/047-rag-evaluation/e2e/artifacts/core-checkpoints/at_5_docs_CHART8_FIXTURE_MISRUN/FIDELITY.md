# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 47 / total_answerable=47
- answer_in_evidence_rate (raw): 0.7447
- answer_in_evidence_rate_long (GATE): **0.7188** (n_long=32, min_needle≥3)
- answer_in_document_rate: 0.8298
- short_needle_fp_suspect: 9 / short_needle=15
- representation_miss_n (raw): 12
- representation_miss_long_n: 9
- retrieval_miss_given_rep_ok_n: 13

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.6 n=10 threshold≥0.5
- Table a_in_e_long: PASS rate=0.5833333333333334 n=12 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.5000 (n=2)
- Chart: rate=0.7143 (n=14)
- Figure: rate=0.8750 (n=8)
- Generalized-text (Layout): rate=1.0000 (n=5)
- Pure-text (Plain-text): rate=0.6667 (n=15)
- Table: rate=0.6250 (n=16)

## By evidence source (long-needle / GATE)
- ?: rate=1.0000 (n=1)
- Chart: rate=0.6000 (n=10)
- Figure: rate=1.0000 (n=3)
- Generalized-text (Layout): rate=1.0000 (n=4)
- Pure-text (Plain-text): rate=0.5455 (n=11)
- Table: rate=0.5833 (n=12)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.5000 (n=2)
- Chart: rate=1.0000 (n=6)
- Figure: rate=0.8571 (n=7)
- Generalized-text (Layout): rate=1.0000 (n=4)
- Pure-text (Plain-text): rate=0.6667 (n=6)
- Table: rate=0.6364 (n=11)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.
