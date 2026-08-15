# 02 — Cross-Ref Matrix

## Spec / doc map

| Spec / Doc | Relationship to SPEC-124 |
|------------|--------------------------|
| [SPEC-018](../018-observability/) | Parent observability SSOT; extend, don’t fork |
| [`docs/OBSERVABILITY.md`](../../docs/OBSERVABILITY.md) | Operator env + Docker; add Langfuse section |
| [SPEC-123](../123-env-config-priority/) | Model priority law; Langfuse stays env-only (orthogonal) |
| [SPEC-046](../046-*) (GenAI spans) | `rag.retrieval` / `rag.generation` helpers live here |
| LightRAG comparison docs | Gap: Python Langfuse vs Rust missing |
| [`.github/skills/langfuse/`](../../.github/skills/langfuse/) | Agent skill for instrumentation + CLI audit |

## Violation / gap register

| ID | Gap | Law | Fix |
|----|-----|-----|-----|
| G1 | OTEL exporter is gRPC-only | LAW-124-1 | Add OTLP/HTTP path |
| G2 | `with_rag_generation_span` unused | LAW-124-7 | Wire query + pipeline |
| G3 | No Langfuse status on health/settings | LAW-124-5,6 | DTO + Settings card |
| G4 | No deep link when configured | LAW-124-6 | Open in Langfuse button |
| G5 | Token usage not on spans | best practices | Record when provider returns usage |
| G6 | Trace attrs not propagated to children | LAW-124-9 | Context / span fields |

## ASCII dependency

```ascii
  SPEC-018 (observability)
       │
       └─ SPEC-124 (this)
              ├─ constrains transport (HTTP for Langfuse)
              ├─ extends health.operational.observability
              ├─ Settings UX (new card)
              └─ does NOT change SPEC-123 model resolve chain
```

## External refs

- https://langfuse.com/integrations/native/opentelemetry
- https://langfuse.com/docs/observability/best-practices
- https://crates.io/crates/opentelemetry-langfuse (optional helper crate)
