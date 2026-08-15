# Lens 007 — AI Engineer

## Stake

Trace quality determines whether Langfuse is useful for prompt iteration and cost control.

## Baseline requirements (from Langfuse skill)

| Requirement | EdgeQuake mapping |
|-------------|-------------------|
| Model name | `gen_ai.request.model` on generation spans |
| Token usage | Record when provider returns usage; else omit |
| Good names | `retrieve-context`, `generate-answer`, `extract-entities`, `embed-chunks` |
| Nesting | retrieval + generation under query/ingest root |
| Observation types | generation vs retrieval via GenAI attrs / span names |
| Sensitive data | `query_preview` truncation; never log API keys |
| Trace I/O | user query in / answer out on root when feasible |

## Sessions

Chat multi-turn: pass `session_id` (conversation id) when available; one trace per turn.

## Audit loop

After wiring: run query → fetch trace via `npx langfuse-cli` → compare to https://langfuse.com/docs/observability/best-practices → fix gaps.

## Cross-refs

- Observability lens: [008-observability.md](008-observability.md)
- Skill: [../../../.github/skills/langfuse/references/instrumentation.md](../../../.github/skills/langfuse/references/instrumentation.md)
