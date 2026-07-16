# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `True`
- n_answerable_audited: 127 / total_answerable=127
- answer_in_evidence_rate (raw): 0.7165
- answer_in_evidence_rate_long (GATE): **0.6974** (n_long=76, min_needle≥3)
- answer_in_document_rate: 0.8661
- short_needle_fp_suspect: 23 / short_needle=51
- representation_miss_n (raw): 36
- representation_miss_long_n: 23
- retrieval_miss_given_rep_ok_n: 23

## Wave 1 gates (long-needle)
- Chart a_in_e_long: PASS rate=0.65 n=20 threshold≥0.5
- Table a_in_e_long: PASS rate=0.5769230769230769 n=26 threshold≥0.55

## By evidence source (raw / multi-label)
- ?: rate=0.6000 (n=5)
- Chart: rate=0.7097 (n=31)
- Figure: rate=0.7778 (n=45)
- Generalized-text (Layout): rate=0.8636 (n=22)
- Pure-text (Plain-text): rate=0.6923 (n=39)
- Table: rate=0.6053 (n=38)

## By evidence source (long-needle / GATE)
- ?: rate=0.6667 (n=3)
- Chart: rate=0.6500 (n=20)
- Figure: rate=0.8182 (n=22)
- Generalized-text (Layout): rate=0.9091 (n=11)
- Pure-text (Plain-text): rate=0.6000 (n=25)
- Table: rate=0.5769 (n=26)

## By evidence source exclusive (len==1, raw)
- ?: rate=0.6000 (n=5)
- Chart: rate=0.9167 (n=12)
- Figure: rate=0.7097 (n=31)
- Generalized-text (Layout): rate=0.8750 (n=8)
- Pure-text (Plain-text): rate=0.7000 (n=10)
- Table: rate=0.6190 (n=21)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
