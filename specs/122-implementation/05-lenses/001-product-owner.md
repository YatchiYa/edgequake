# Lens 001 — Product Owner

## Stake

Partners file “bulk upload bug” when capacity law and async UX collide ([#361](https://github.com/raphaelmansuy/edgequake/issues/361) / [#365](https://github.com/raphaelmansuy/edgequake/issues/365)). Closing without honesty or measurement burns trust; “just parallelize” burns reliability.

## Outcome

| Priority | Outcome |
|----------|---------|
| P0 | Publish concurrency truth + admit≠ready language |
| P0 | Measured Ollama vs Mistral docs/min baselines |
| P1 | Optional provider-aware concurrency toward agreed SLO |
| P2 | PDF cost controls only if H2/H3 proven |
| Later | Progressive partial search (out of v1 unless funded) |

## Acceptance language

> “When I upload many documents, EdgeQuake tells me they are queued and shows processing progress. I understand local LLM mode processes roughly one document at a time, while Docker/cloud can process several in parallel. Documents become searchable when processing completes — not merely when the upload finishes.”

## Non-goals

- Marketing “instant bulk RAG” on local Ollama
- Closing issues as “fixed” without Phase A evidence

## Cross-refs

- WHY: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
