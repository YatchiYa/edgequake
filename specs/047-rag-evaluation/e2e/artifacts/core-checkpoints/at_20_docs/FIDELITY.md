# SPEC-047 core — W1 representation fidelity

- protocol: `026-listmem-2026-07-15`
- gateable: `False` (audited 0/146 answerable (errors=146) — cross-run compare only if peer has same n)
- n_answerable_audited: 0 / total_answerable=146
- answer_in_evidence_rate (raw): 0.0000
- answer_in_evidence_rate_long (GATE): **None** (n_long=0, min_needle≥None)
- answer_in_document_rate: 0.0000
- short_needle_fp_suspect: 0 / short_needle=0
- representation_miss_n (raw): 0
- representation_miss_long_n: 0
- retrieval_miss_given_rep_ok_n: 0

## Wave 1 gates (long-needle)
- Chart a_in_e_long: FAIL rate=None n=None threshold≥None
- Table a_in_e_long: FAIL rate=None n=None threshold≥None

## By evidence source (raw / multi-label)

## By evidence source (long-needle / GATE)

## By evidence source exclusive (len==1, raw)

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve). Gate Wave 1 on long-needle rates, not raw short-needle rates.

Do not compare raw a_in_e across runs with different n_answerable_audited.

Wrote /Users/raphaelmansuy/Github/03-working/edgequake/specs/047-rag-evaluation/e2e/artifacts/core/fidelity.json
