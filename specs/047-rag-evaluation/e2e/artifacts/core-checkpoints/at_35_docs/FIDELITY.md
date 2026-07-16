# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 248 / total_answerable=248
- answer_in_evidence_rate (raw): 0.7137
- answer_in_evidence_rate_long (GATE): **0.6831** (n_long=142, min_needle≥3)
- answer_in_document_rate: 0.8831
- short_needle_fp_suspect: 61 / short_needle=106
- representation_miss_n (raw): 71
- representation_miss_long_n: 45
- retrieval_miss_given_rep_ok_n: 59

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.59375 n=32 threshold≥0.5
- Table a_in_e_long: PASS rate=0.6491228070175439 n=57 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.6250 (n=8)
- Chart: rate=0.7091 (n=55)
- Figure: rate=0.7320 (n=97)
- Generalized-text (Layout): rate=0.8056 (n=36)
- Pure-text (Plain-text): rate=0.7176 (n=85)
- Table: rate=0.6667 (n=87)

## By evidence source (long-needle / GATE)
- ?: rate=0.6000 (n=5)
- Chart: rate=0.5938 (n=32)
- Figure: rate=0.7551 (n=49)
- Generalized-text (Layout): rate=0.8235 (n=17)
- Pure-text (Plain-text): rate=0.6296 (n=54)
- Table: rate=0.6491 (n=57)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.6250 (n=8)
- Chart: rate=0.8261 (n=23)
- Figure: rate=0.7119 (n=59)
- Generalized-text (Layout): rate=0.8750 (n=8)
- Pure-text (Plain-text): rate=0.7000 (n=10)
- Table: rate=0.6667 (n=45)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
