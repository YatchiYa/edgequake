# SPEC-047 smoke — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 75 / total_answerable=75
- answer_in_evidence_rate (raw): 0.7067
- answer_in_evidence_rate_long (GATE): **0.6596** (n_long=47, min_needle≥3)
- answer_in_document_rate: 0.8267
- short_needle_fp_suspect: 14 / short_needle=28
- representation_miss_n (raw): 22
- representation_miss_long_n: 16
- retrieval_miss_given_rep_ok_n: 16

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.5714285714285714 n=14 threshold≥0.5
- Table a_in_e_long: PASS rate=0.5882352941176471 n=17 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.5000 (n=2)
- Chart: rate=0.6364 (n=22)
- Figure: rate=0.8095 (n=21)
- Generalized-text (Layout): rate=0.8182 (n=11)
- Pure-text (Plain-text): rate=0.6538 (n=26)
- Table: rate=0.6250 (n=24)

## By evidence source (long-needle / GATE)
- ?: rate=1.0000 (n=1)
- Chart: rate=0.5714 (n=14)
- Figure: rate=0.7778 (n=9)
- Generalized-text (Layout): rate=0.8571 (n=7)
- Pure-text (Plain-text): rate=0.5294 (n=17)
- Table: rate=0.5882 (n=17)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.5000 (n=2)
- Chart: rate=0.8571 (n=7)
- Figure: rate=0.8125 (n=16)
- Generalized-text (Layout): rate=1.0000 (n=4)
- Pure-text (Plain-text): rate=0.5714 (n=7)
- Table: rate=0.6000 (n=15)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.
