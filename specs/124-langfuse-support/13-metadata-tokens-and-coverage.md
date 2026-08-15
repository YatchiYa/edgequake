# 13 — Metadata: tokens yes, cost never + pipeline coverage

## Intent

Honest GenAI coverage for Query and Ingest: Langfuse receives **token usage** and **observation types**, never **cost attributes** from EdgeQuake.

Official Langfuse OTEL mapping accepts session/user, `langfuse.observation.type`, model, `gen_ai.usage.*` / `llm.token_count.*`, optional I/O previews, **and** cost via `gen_ai.usage.cost` / `langfuse.observation.cost_details`. EdgeQuake **uses** usage; **forbids** cost (LAW-124-12).

Langfuse may still **compute** USD from model pricing tables when it sees model + tokens — that is platform-side inference, not EdgeQuake emitting cost attrs.

## Allowlist vs denylist

| Kind | Keys | Policy |
|------|------|--------|
| Usage | `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens` (i64) | Emit when provider returns counts |
| Observation type | `langfuse.observation.type` = `generation` \| `retriever` \| `embedding` \| `chain` | Always set on helper spans |
| Session / user | `langfuse.session.id`, `gen_ai.conversation.id`, … | See [12-sessions-and-genai.md](12-sessions-and-genai.md) |
| I/O preview | `langfuse.observation.input` / `output` (truncated) | Optional; LAW-124-8 |
| Feature tag | `langfuse.trace.tags` = `query` \| `ingest` | On HTTP / ingest roots |
| **Cost** | `gen_ai.usage.cost`, `langfuse.observation.cost_details`, `langfuse.observation.cost` | **Never emit** (`COST_ATTR_DENYLIST`) |

SSOT: `record_gen_ai_usage` / `record_observation_io` in `edgequake-observability` (LAW-124-14/16). Call sites never invent attribute strings.

**I/O mapping (LAW-124-16..18):** Langfuse UI Input/Output requires `langfuse.observation.input`/`output` (aliases `gen_ai.prompt`/`gen_ai.completion`). `gen_ai.retrieval.query.text` alone is **not** enough — see [14-observation-io-and-full-observe.md](14-observation-io-and-full-observe.md).

## Query span tree

```ascii
  query / query_stream / chat_*          (HTTP root + tags=query + session?)
    ├─ rag.retrieval                    type=retriever
    ├─ rag.generation extract-keywords  type=generation + usage
    └─ rag.generation generate-answer   type=generation + usage
         (or generate-bypass-answer)
```

Stream path (WebUI primary) wraps `generate-answer` the same as sync.

## Ingest span tree

```ascii
  ingest.document                       type=chain, tags=ingest
    ├─ rag.generation extract-entities  + usage
    ├─ rag.generation summarize-*       + usage
    └─ rag.embedding embed-chunks       type=embedding
```

Pipeline product USD (`progress/cost.rs`) is **not** exported to OTEL.

## Token source

| Call | Source |
|------|--------|
| Sync/stream answer, bypass, keywords, extract, summarize | `LLMResponse.prompt_tokens` / `completion_tokens` |
| True token `stream()` without final usage | Omit usage attrs (Empty) |
| Embeddings | Span only; input tokens if/when provider exposes them |

## Operator live smoke

1. Configure Langfuse keys; run a `/query` or chat turn.
2. Open Langfuse → Trace → generation observation → **Usage** shows input/output tokens.
3. Confirm EdgeQuake did not send cost attrs (CI denylist + in-memory exporter tests). Cost column in UI may still fill from Langfuse model pricing — that is OK.

## CI unfakable proof (LAW-124-15)

- Unit: denylist + helper source scan.
- Integration: `SdkTracerProvider` + `InMemorySpanExporter` asserts usage ints and **absence** of cost keys — **no skip** for missing Langfuse credentials.
- Contract grep: stream path contains `with_rag_generation_span`.

Playwright live session/settings specs remain optional when keys absent; they are **not** the only gate.
