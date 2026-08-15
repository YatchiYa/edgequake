# Lens 001 — Product Owner

## Stake

Operators and AI engineers need to **trust and debug** RAG quality. Without Langfuse, EdgeQuake loses a competitive parity point vs LightRAG and slows incident MTTR.

## Outcomes (v1)

1. Optional Langfuse export (env) — zero friction when unused.
2. Settings discovers config + opens Langfuse UI.
3. Nested traces for query + ingest — enough to answer “what did the model see?”
4. No secrets in product DB — compliance-friendly.

## Non-outcomes (v1)

Prompt versioning, eval suites, human annotation workflows — later specs.

## Acceptance narrative

> As an operator, I set three env vars, rebuild with `otel`, restart, open Settings, click **Open in Langfuse**, run a query, and see a retrieval + generation tree.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- Marketing: [006-marketing-growth.md](006-marketing-growth.md)
