# 14 — Observation I/O and full observe (Langfuse Aug 2026)

## Honest assessment (live UI + docs)

**Working:** OTLP/HTTP export; observation types (`generation` / `embedding` / `retriever` / `chain`); sessions; tenant/workspace metadata; ingest tree shape; Langfuse-mapped Input/Output on GenAI + key workflow spans (I6); gleaning generations (I7).

**Was broken (pre-I6):** Langfuse UI Input/Output on retriever and many workflow spans showed `null` / `undefined` because only `gen_ai.retrieval.query.text` was set.

Langfuse maps observation I/O **only** from ([empty I/O FAQ](https://langfuse.com/faq/all/empty-trace-input-and-output) / OTEL property mapping):

| Field | Accepted OTEL attributes (priority) |
|-------|-------------------------------------|
| input | `langfuse.observation.input`, `gen_ai.prompt`, `input.value` |
| output | `langfuse.observation.output`, `gen_ai.completion`, `output.value` |

`gen_ai.retrieval.query.text` is **not** mapped → retriever Input stayed null until I6 dual-wrote Langfuse keys.

Trace-level I/O is **deprecated** in Langfuse v4 — put overall request/response on the **root observation**.

## Gap table (screenshot-aligned)

| Span | Type | Before I6 | After |
|------|------|-----------|-------|
| `retrieval edgequake` | retriever | Input null | `langfuse.observation.input` = query; output = counts JSON |
| `generate-answer` / extract / summarize / glean | generation | Often truncated I/O via helper | Keep + dual-write `gen_ai.prompt`/`completion` |
| `embed-chunks` | embedding | No I/O | input = kind/count; output = vector count/dim |
| `ingest.document` | chain | No I/O | input = doc id + content preview; output = stats |
| `pipeline_chunk_extraction` | span | Bare instrument, null I/O | type + chunk_count in / success-fail out |
| Query/chat HTTP root | span | Session only | query + answer preview on root |

**Still never:** `gen_ai.usage.cost`, `langfuse.observation.cost_details`, product USD.

**Residual:** live UI re-smoke after deploy; TraceId unification for per-trace deep-links.

## Laws

| ID | Law |
|----|-----|
| LAW-124-16 | **Observation I/O SSOT** — only `record_observation_io` / span helpers set input/output keys |
| LAW-124-17 | **Map what Langfuse reads** — `langfuse.observation.*` + dual-write `gen_ai.prompt`/`gen_ai.completion` |
| LAW-124-18 | **GenAI-typed + key workflow spans** get truncated I/O; noise middleware SPANs stay metadata-only |

## Target I/O matrix

```ascii
  query / chat / stream root     input=query preview  output=answer preview
    ├─ rag.retrieval             input=query          output={empty,chunks,entities}
    ├─ extract-keywords          input=query          output=keywords preview
    └─ generate-answer           input=query/prompt   output=answer (+ usage)

  ingest.document                input={doc_id,preview} output={chunks,entities,…}
    ├─ pipeline_chunk_extraction input={chunk_count}  output={ok,fail,entities}
    ├─ extract-entities          generation I/O + usage
    ├─ extract-entities-glean    generation I/O + usage (I7)
    └─ embed-chunks              input={kind,n}       output={vectors,dim}
```

## Operator smoke

1. Run a Mix query with session → open **retriever** → Input shows query; Output shows counts JSON.
2. Ingest a document → open `ingest.document` / `pipeline_chunk_extraction` / glean → Input/Output non-null.
3. Confirm no cost attrs on span metadata.
4. Chat turn → **Open session in Langfuse** in message metadata when export is active.

## CI proof

`make spec124-proof` — InMemorySpanExporter asserts `langfuse.observation.input` / `output`, session bind, `with_llm_generation`; contract greps for stream + gleaning.

## Cross-refs

- Tokens / cost: [13-metadata-tokens-and-coverage.md](13-metadata-tokens-and-coverage.md)
- Sessions: [12-sessions-and-genai.md](12-sessions-and-genai.md)
- Assessment: [11-honest-assessment.md](11-honest-assessment.md)
