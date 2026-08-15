# 00 — Why SPEC-124

## Trigger

EdgeQuake runs a full RAG loop (retrieve → generate → extract → embed) with Prometheus counters and optional Jaeger OTLP, but operators cannot open a single LLM-native trace tree for a failing query or a costly ingest. LightRAG already integrates Langfuse; EdgeQuake docs call this out as a gap.

## Product WHY

```ascii
  Operator: “Why did this answer hallucinate / cost $X / take 12s?”
       │
       ▼
  Today:
       Prometheus: aggregates, no prompt I/O tree
       Jaeger (optional): generic spans, gRPC only
       Generation helper: defined, NEVER called
              │
              ▼
  Blind spot: no Langfuse / LLM UI; no “Open in Langfuse” from Settings
```

## Five WHYs

1. **Why can’t we debug an LLM call end-to-end?** No LLM-observability SaaS export.
2. **Why not point existing OTEL at Langfuse?** Exporter uses `with_tonic()` (gRPC); Langfuse accepts **HTTP only**.
3. **Why are generation spans missing?** `with_rag_generation_span` exists but call sites never use it.
4. **Why no Settings link?** No status DTO / UI card for optional Langfuse config.
5. **Root cause:** Observability stack optimized for ops metrics, not GenAI product debugging — and transport mismatch blocks the obvious Langfuse path.

## Job to be done

> When Langfuse is configured via env, every query turn and ingest job emits a nested, named trace I can open from Settings (and later from the query result), without storing secrets in the app DB.

## Success criteria

1. OTLP/HTTP export to Langfuse when `LANGFUSE_PUBLIC_KEY` + `LANGFUSE_SECRET_KEY` set (and `otel` feature built).
2. Nested spans: HTTP → query/ingest → retrieval → generation (stable names).
3. Settings card: status + env snippets + **Open in Langfuse** iff configured.
4. Secrets never leave env; UI never shows secret values.
5. Edge-case matrix tested (misconfig, down, dual export, no keys).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
