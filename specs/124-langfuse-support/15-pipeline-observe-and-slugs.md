# 15 — Pipeline observe and slugs (I8)

## Intent

Operators must filter Langfuse by **human slugs** without losing **UUID identity**, and must see **what the query and ingest pipelines actually did** (mode, fusion, rerank, parser, vision pass) — not only LLM I/O.

## Laws

| ID | Law |
|----|-----|
| LAW-124-19 | **Slugs additive to GUIDs** — emit `langfuse.trace.metadata.tenant_slug` / `workspace_slug` **and** keep `tenant_id` / `workspace_id`. Never copy a slug into an `*_id` field. Omit blank slugs; never invent. |
| LAW-124-20 | **Filterable metadata prefix** — Langfuse-queryable keys use only `langfuse.trace.metadata.<key>` or `langfuse.observation.metadata.<key>`. Keys after the prefix are `[a-z0-9_]+`. Values truncated to 200 chars. Arbitrary `rag.*` attrs may remain for Jaeger; they are **not** the Langfuse filter contract. |
| LAW-124-21 | **Query pipeline meta SSOT** — `QueryPipelineMeta` recorded once from `QueryStats` via observability helpers. Call sites never invent metadata key strings. |
| LAW-124-22 | **Ingest stages** — converting / pass_a / pass_b / chunking / extract / embed / persist are typed observations. Ingest Langfuse session = durable `document_id`. Parse/vision/kg stats via `record_ingest_parse_meta` / `record_ingest_kg_meta`. |

## Identity (additive)

| Key | Value |
|-----|-------|
| `langfuse.trace.metadata.tenant_id` | UUID (existing) |
| `langfuse.trace.metadata.workspace_id` | UUID (existing) |
| `langfuse.trace.metadata.tenant_slug` | Tenant.slug when resolved |
| `langfuse.trace.metadata.workspace_slug` | Workspace.slug when resolved |

Query: resolve from `Workspace` + `get_tenant` (fail-open on slug lookup). Ingest: same, plus `session.id` = `document_id`.

## Query tree

```ascii
query root  tags=query  ids+slugs
  ├─ extract-keywords | keyword-cache | keyword-heuristic
  ├─ query.embed                 type=embedding
  ├─ rag.retrieval               type=retriever (per arm / mode)
  ├─ query.fuse                  mix/hybrid
  ├─ query.rerank                applied|skipped
  └─ generate-answer | generate-bypass-answer | answer-cache
```

Trace metadata from QueryStats: `mode`, `query_intent`, `fusion`, `arms_run`, `keyword_cache_hit`, `answer_cache_hit`, `reasoning_effort`, `sparse_outcome`, `citation_count`, `context_empty`, `context_truncated`.

## Ingest tree

```ascii
ingest.task  type=chain  tags=ingest  session=document_id  ids+slugs
  ├─ ingest.converting           parser + fallback
  │    ├─ ingest.pass_a          pages, vision model
  │    └─ ingest.pass_b          figures; pdf-pass-b-figure generations
  ├─ ingest.document             KG slice
  │    ├─ ingest.chunking
  │    ├─ pipeline_chunk_extraction
  │    │    extract-entities / extract-entities-glean
  │    └─ embed-chunks
  ├─ ingest.persist
  └─ summarize-*
```

I/O stays truncated counts/previews (LAW-124-8). Never dump page images, markdown bodies, or chunk text.

## CI

`make spec124-proof` asserts slugs co-emitted with GUIDs, query pipeline meta keys, ingest parse meta, no cost attrs.

## Cross-refs

- Laws index: [01-first-principles.md](01-first-principles.md)
- Trees: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
