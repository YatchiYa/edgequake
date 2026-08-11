# SPEC-122 — Bulk Upload / Ingest Latency

> **Mission:** Explain and measure why multi-document upload/ingest feels excessively slow ([#361](https://github.com/raphaelmansuy/edgequake/issues/361) / [#365](https://github.com/raphaelmansuy/edgequake/issues/365)), separate capacity law from logic bugs, and ship capacity-governed UX + optional concurrency tuning — never unbounded parallel LLM.  
> **Trigger:** Partner reports on Docker **v0.12.11** → **v0.24.1**: bulk uploads take too long; docs stay Processing.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | N-file batch: long wall clock; Processing until KG insert completes |
| Classification | **Capacity / LLM+vision throughput + expectation gap** — not a single logic defect |
| Still on HEAD? | **Yes as behavior** under local clamps; **perceived latency** under Docker/Mistral (O(chunks)×RTT) |
| PDF quality link? | **Not primary serial cause.** Vision path taxes PDF bulk; chunk inflation can amplify extract/embed |
| Fix posture | Measure → honest UX/SLO (P0) → provider-aware concurrency (P1) → PDF cost if proven (P2) |

```ascii
  WebUI (≤3 parallel admits)
       │
       ▼
  API 202 + durable task row
       │
       ▼
  Worker pool
       │
       ├─ MAX_TASKS_PER_TENANT (local=1 / Docker=6 / cloud=12)
       ├─ PDF: PdfProcessing → Insert (two tasks)
       │     └─ PDF_VISION_JOBS × PDF_CONCURRENCY
       └─ Insert: extract × embed (semaphores)
              │
              ▼
         Searchable only after Insert completes
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-122-1..10)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, system, reliability, AI)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-reproduction
   → 11-honest-assessment
   → 12-pdf-quality-hypothesis
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| R1 | Arm A Ollama bulk measurement | Done (N=5 → 59.2 s, 5.06 docs/min, tenant=1) |
| R2 | Arm B Mistral bulk measurement | Done (N=5 → 45.0 s, 6.66 docs/min, tenant=6) |
| H* | Hypotheses H1–H5 accept/reject | Done (H1 accept; H2 partial; H3/H4/H5 reject) |
| I1 | Phase A UX/SLO/FAQ/docs | Done (FAQ + quick-start + perf-tuning + harness + P0 UI) |
| I2 | Phase B concurrency (gated) | Deferred (modest Arm B gain; no unbounded raise) |
| I3 | Phase C PDF cost (gated) | Deferred (H2 partial only) |
| G1 | GitHub #361/#365 update | Done |
| A1 | Acceptance | Open (partner ack / SLO) |

## Related

- [#361](https://github.com/raphaelmansuy/edgequake/issues/361), [#365](https://github.com/raphaelmansuy/edgequake/issues/365)
- [`specs/111-issues/issue-361-bulk-upload.md`](../111-issues/issue-361-bulk-upload.md)
- [SPEC-090](../090-performance/) — DB counter / claim_next latency
- [SPEC-091](../091-simplify-data-layer/) — queue admission first principles
- [SPEC-057](../057-pipeline-reliability/) — fairness park, convert→ingest
- [SPEC-098](../098-ux-ui-improvement/) / GH-350 — WebUI N× admits vs `/upload/batch`
- [SPEC-121](../121-pdf-docx/) — format matrix (orthogonal; PDF path exists)
- Ops: [`docs/operations/performance-tuning.md`](../../docs/operations/performance-tuning.md)
- Ollama concurrency: [docs.ollama.com/faq](https://docs.ollama.com/faq) (`OLLAMA_NUM_PARALLEL`)

## Non-goals (v1)

- Unbounded parallel extract/embed/vision
- Rewriting PDF vision “for quality” alone (see [12-pdf-quality-hypothesis.md](12-pdf-quality-hypothesis.md))
- Office/DOCX ingest (SPEC-121)
- Closing #361/#365 without measured SLO + partner-facing honesty
