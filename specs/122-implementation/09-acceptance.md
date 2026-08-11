# 09 — Acceptance

## Product checklist

- [x] Concurrency SSOT published (FAQ + SPEC-122 + ops)
- [x] Partner-facing language: admit ≠ searchable; local vs Docker explained
- [x] Documents P0 UI: admit toast + bulk banner + serial/parallel hint (SPEC-122 Phase A)
- [x] #361 and #365 updated with SPEC-122 link + measurement summary
- [x] Issues remain open until partner ack **or** Phase A shipped and SLO documented
- [x] No claim of “fixed bulk speed” without Phase B gate evidence

## Technical checklist

- [x] Arm A (Ollama) timings recorded
- [x] Arm B (Mistral) timings recorded
- [x] Arm C baseline recorded
- [x] H1–H5 accepted or rejected with evidence
- [x] Measurement harness present and documented
- [x] `admit-copy` unit tests (U1/U2) + Playwright `spec122-admit-honesty`
- [x] Phase B/C not merged without gates

## Process checklist

- [x] Cross-ref pack complete (G0)
- [x] Task log written under `/logs/`
- [x] DRY: no divergent concurrency tables in new docs
- [x] SPEC-090 / SPEC-121 boundaries respected
- [x] SOLID: admit-copy SSOT + status-domain; banner is presentation adapter

## Close criteria for GitHub

Close #361/#365 only when:

1. Phase A complete (honest UX/docs + measurements), **and**
2. Either partner accepts capacity explanation **or** Phase B meets an agreed docs/min SLO, **and**
3. Acceptance language below is true in product.

## Acceptance language

> “When I upload many documents, EdgeQuake tells me they are queued and shows processing progress. I understand local LLM mode processes roughly one document at a time, while Docker/cloud can process several in parallel. Documents become searchable when processing completes — not merely when the upload finishes. Operators can inspect queue metrics and tune concurrency safely.”

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
