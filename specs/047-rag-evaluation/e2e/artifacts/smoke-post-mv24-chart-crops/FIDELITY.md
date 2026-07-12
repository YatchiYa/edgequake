# SPEC-047 smoke — W1 representation fidelity (post MV-24)

workspace: `17d517a4-595c-4e45-b9ab-ad6b042a2461`

- n_answerable_audited: 75
- answer_in_evidence_rate: **0.5333**
- answer_in_document_rate: 0.6667
- Chart answer_in_evidence: **0.4091** (n=22) ← G-A ≥0.50 **FAIL** (unchanged vs MV-18)
- Chart answer_in_document: **0.5455** (page-local dump gap remains)
- representation_miss_n: 35
- retrieval_miss_given_rep_ok_n: 8

## Acc (query-only, document-scope)

| Metric | MV-18 | MV-24 | Δ |
|--------|-------|-------|---|
| Acc | 0.423 | **0.433** | +0.010 |
| F1 | 0.232 | **0.262** | +0.030 |
| Chart Acc | 0.182 | 0.182 | 0 |
| Chart a_in_e | 0.409 | 0.409 | 0 (G-A FAIL) |
| page_hit@5 | 0.72 | **0.80** | +0.08 |
| Unans Acc | 0.786 | 0.738 | -0.05 |

MV-24 crop writes: **8/8** docs (live: political-release 15 crops / 17 pages; 2311 paper 53 crops / 117 pages).

## Verdict

Crops + hi-res re-render **shipped and fired**, but Chart gold strings on evidence pages did not increase. Next lawful levers: **MV-26/27** (routing + specialize soft-fail) and **MV-28** (page-local dump — a_in_doc − a_in_e gap).

If answer not in evidence-page markdown → W1 (ingest). If answer in markdown but page_hit@5 false → W2 (retrieve).
