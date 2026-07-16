# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 183 / total_answerable=183
- answer_in_evidence_rate (raw): 0.7377
- answer_in_evidence_rate_long (GATE): **0.7308** (n_long=104, min_needle≥3)
- answer_in_document_rate: 0.8907
- short_needle_fp_suspect: 42 / short_needle=79
- representation_miss_n (raw): 48
- representation_miss_long_n: 28
- retrieval_miss_given_rep_ok_n: 43

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.6538461538461539 n=26 threshold≥0.5
- Table a_in_e_long: PASS rate=0.6571428571428571 n=35 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.5714 (n=7)
- Chart: rate=0.7381 (n=42)
- Figure: rate=0.7681 (n=69)
- Generalized-text (Layout): rate=0.8519 (n=27)
- Pure-text (Plain-text): rate=0.7458 (n=59)
- Table: rate=0.6607 (n=56)

## By evidence source (long-needle / GATE)
- ?: rate=0.5000 (n=4)
- Chart: rate=0.6538 (n=26)
- Figure: rate=0.8235 (n=34)
- Generalized-text (Layout): rate=0.9167 (n=12)
- Pure-text (Plain-text): rate=0.6486 (n=37)
- Table: rate=0.6571 (n=35)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.5714 (n=7)
- Chart: rate=0.9375 (n=16)
- Figure: rate=0.7083 (n=48)
- Generalized-text (Layout): rate=0.8750 (n=8)
- Pure-text (Plain-text): rate=0.7000 (n=10)
- Table: rate=0.6970 (n=33)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
