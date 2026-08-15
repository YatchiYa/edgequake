# Lens 008 — Observability

## Stake

Langfuse must compose with existing Prometheus + optional Jaeger without double-counting pain or blocking the hot path.

## Design rules

```ascii
  Metrics (Prometheus)  → counters/histograms for SLOs
  Traces (Jaeger gRPC)  → infra / request topology
  Traces (Langfuse HTTP)→ GenAI I/O, cost, eval readiness
```

## Reliability

- BatchSpanProcessor + Tokio runtime
- Export errors → log + metric; never fail Axum handler
- Shutdown flush via `ObservabilityGuard`
- RUST_LOG / EnvFilter bounds what reaches OTEL (SPEC-083 D-46)

## Noise control

Do not auto-instrument sqlx or every HTTP client span for Langfuse. Prefer explicit RAG helpers.

## Misconfiguration signals

| Condition | Signal |
|-----------|--------|
| Keys without `otel` feature | Boot warn + health `otel_feature_built=false` |
| Partial keys | `enabled=false`, requirements unsatisfied |
| Endpoint 401 | Export error counter; app healthy |

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Edges: [../10-edge-cases.md](../10-edge-cases.md)
- SPEC-018: [../../018-observability/](../../018-observability/)
