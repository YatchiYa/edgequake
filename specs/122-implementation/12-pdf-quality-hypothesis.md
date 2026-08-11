# 12 — PDF Quality vs Throughput Hypothesis

## Question

Can slow bulk ingest be blamed on **PDF parsing quality**?

## First Principles split

```ascii
  “PDF quality” blends three different costs:

  Q1  Vision/quality path latency
      = pages × VLM RTT × (1 / page_concurrency) × job_slots
      → product chose quality convert (pdfium + vision)

  Q2  Chunk inflation
      = verbose/noisy markdown → more chunks → more extract+embed calls
      → quality failure mode that looks like “slow”

  Q3  Wrong-format / admit failures
      = SPEC-121 territory (not this SPEC)
```

## Linkage to #361/#365

| If true… | Then bulk text-only would… | Phase |
|----------|----------------------------|-------|
| Serial fairness (H1) | Also be slow (linear in N) | A/B |
| Vision tax (H2) | Be much faster than PDF bulk | C |
| Chunk inflation (H3) | Be faster; PDF shows huge chunk counts | C |
| DB contention (H4) | Show store_contention with idle LLM | SPEC-090 |
| Transfer bound (H5) | Finish processing almost when upload finishes | A UX only |

**Normative claim (LAW-122-6):** PDF vision cost is a **quality-path tax**, not evidence that PDF is unsupported or that parse quality is “broken.” Poor markdown can **amplify** extract cost (H3) but does not explain local tenant=1 serial drain for text files.

## Decision record

| ID | Outcome | Notes |
|----|---------|-------|
| H2 | **PARTIAL** | Qwen.pdf (1 page): convert 11.5 s vision; insert → 5 chunks. Quality-path tax confirmed; not root of text bulk serial drain (Arm A). Multi-page fleets still justify Phase C ETA/cost UX. |
| H3 | **REJECT** | No chunk inflation on this fixture (5 chunks / 3028 md chars). |
| Phase C trigger | **Deferred** | Full Accept on H2/H3 not met; ship Phase A honesty + optional multi-page ETA later |

## Non-goals

- Changing SPEC-121 format matrix
- Replacing vision convert with lossy text-only without product approval
- Treating every slow PDF as a quality defect

## Cross-refs

- Repro: [10-reproduction.md](10-reproduction.md)
- SPEC-121: [../121-pdf-docx/README.md](../121-pdf-docx/README.md)
- PDF processing: `edgequake-api/src/processor/pdf_processing.rs`
