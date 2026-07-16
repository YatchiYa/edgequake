# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 53 / total_answerable=53
- answer_in_evidence_rate (raw): 0.7736
- answer_in_evidence_rate_long (GATE): **0.7576** (n_long=33, min_needle≥3)
- answer_in_document_rate: 0.8868
- short_needle_fp_suspect: 12 / short_needle=20
- representation_miss_n (raw): 12
- representation_miss_long_n: 8
- retrieval_miss_given_rep_ok_n: 13

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=1.0 n=5 threshold≥0.5
- Table a_in_e_long: PASS rate=0.7 n=10 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=1.0000 (n=1)
- Chart: rate=0.8571 (n=7)
- Figure: rate=0.8000 (n=20)
- Generalized-text (Layout): rate=1.0000 (n=5)
- Pure-text (Plain-text): rate=0.5556 (n=9)
- Table: rate=0.6875 (n=16)

## By evidence source (long-needle / GATE)
- ?: rate=1.0000 (n=1)
- Chart: rate=1.0000 (n=5)
- Figure: rate=0.7500 (n=8)
- Generalized-text (Layout): rate=1.0000 (n=4)
- Pure-text (Plain-text): rate=0.4286 (n=7)
- Table: rate=0.7000 (n=10)

## By evidence source exclusive (len==1, raw)
- ?: rate=1.0000 (n=1)
- Chart: rate=1.0000 (n=6)
- Figure: rate=0.8125 (n=16)
- Generalized-text (Layout): rate=1.0000 (n=4)
- Pure-text (Plain-text): rate=0.5714 (n=7)
- Table: rate=0.7143 (n=14)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
