# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 91 / total_answerable=91
- answer_in_evidence_rate (raw): 0.7473
- answer_in_evidence_rate_long (GATE): **0.7170** (n_long=53, min_needle≥3)
- answer_in_document_rate: 0.8681
- short_needle_fp_suspect: 20 / short_needle=38
- representation_miss_n (raw): 23
- representation_miss_long_n: 15
- retrieval_miss_given_rep_ok_n: 19

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.6428571428571429 n=14 threshold≥0.5
- Table a_in_e_long: PASS rate=0.65 n=20 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.6667 (n=3)
- Chart: rate=0.6818 (n=22)
- Figure: rate=0.8214 (n=28)
- Generalized-text (Layout): rate=0.8750 (n=16)
- Pure-text (Plain-text): rate=0.6452 (n=31)
- Table: rate=0.6552 (n=29)

## By evidence source (long-needle / GATE)
- ?: rate=1.0000 (n=1)
- Chart: rate=0.6429 (n=14)
- Figure: rate=0.8182 (n=11)
- Generalized-text (Layout): rate=0.8750 (n=8)
- Pure-text (Plain-text): rate=0.5500 (n=20)
- Table: rate=0.6500 (n=20)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.6667 (n=3)
- Chart: rate=1.0000 (n=7)
- Figure: rate=0.8000 (n=20)
- Generalized-text (Layout): rate=1.0000 (n=5)
- Pure-text (Plain-text): rate=0.6667 (n=9)
- Table: rate=0.6875 (n=16)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
