# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 218 / total_answerable=218
- answer_in_evidence_rate (raw): 0.7202
- answer_in_evidence_rate_long (GATE): **0.7016** (n_long=124, min_needle≥3)
- answer_in_document_rate: 0.8899
- short_needle_fp_suspect: 51 / short_needle=94
- representation_miss_n (raw): 61
- representation_miss_long_n: 37
- retrieval_miss_given_rep_ok_n: 50

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.6206896551724138 n=29 threshold≥0.5
- Table a_in_e_long: PASS rate=0.6444444444444445 n=45 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.5714 (n=7)
- Chart: rate=0.7255 (n=51)
- Figure: rate=0.7412 (n=85)
- Generalized-text (Layout): rate=0.8182 (n=33)
- Pure-text (Plain-text): rate=0.7042 (n=71)
- Table: rate=0.6479 (n=71)

## By evidence source (long-needle / GATE)
- ?: rate=0.5000 (n=4)
- Chart: rate=0.6207 (n=29)
- Figure: rate=0.7674 (n=43)
- Generalized-text (Layout): rate=0.8667 (n=15)
- Pure-text (Plain-text): rate=0.6047 (n=43)
- Table: rate=0.6444 (n=45)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.5714 (n=7)
- Chart: rate=0.9000 (n=20)
- Figure: rate=0.7273 (n=55)
- Generalized-text (Layout): rate=0.8750 (n=8)
- Pure-text (Plain-text): rate=0.7000 (n=10)
- Table: rate=0.6757 (n=37)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
