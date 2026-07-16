# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 278 / total_answerable=278
- answer_in_evidence_rate (raw): 0.7158
- answer_in_evidence_rate_long (GATE): **0.6667** (n_long=159, min_needle≥3)
- answer_in_document_rate: 0.8669
- short_needle_fp_suspect: 73 / short_needle=119
- representation_miss_n (raw): 79
- representation_miss_long_n: 53
- retrieval_miss_given_rep_ok_n: 63

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.6 n=35 threshold≥0.5
- Table a_in_e_long: PASS rate=0.5846153846153846 n=65 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.6250 (n=8)
- Chart: rate=0.7069 (n=58)
- Figure: rate=0.7568 (n=111)
- Generalized-text (Layout): rate=0.8108 (n=37)
- Pure-text (Plain-text): rate=0.7158 (n=95)
- Table: rate=0.6400 (n=100)

## By evidence source (long-needle / GATE)
- ?: rate=0.6000 (n=5)
- Chart: rate=0.6000 (n=35)
- Figure: rate=0.7636 (n=55)
- Generalized-text (Layout): rate=0.8235 (n=17)
- Pure-text (Plain-text): rate=0.6167 (n=60)
- Table: rate=0.5846 (n=65)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.6250 (n=8)
- Chart: rate=0.8400 (n=25)
- Figure: rate=0.7353 (n=68)
- Generalized-text (Layout): rate=0.8750 (n=8)
- Pure-text (Plain-text): rate=0.7273 (n=11)
- Table: rate=0.6415 (n=53)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
