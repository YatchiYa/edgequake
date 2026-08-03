---
title: 'Crate Reference'
description: EdgeQuake is organized into 11 focused Rust crates, each with a single responsibility.
---

## EdgeQuake Crate Architecture

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../../ingestion-cancel-and-fairness.md)

> EdgeQuake ships **11 workspace crates** under `edgequake/crates/`. There is no separate `edgequake-llm` or `edgequake-graph` crate; LLM providers and graph logic live inside `edgequake-core`, `edgequake-pipeline`, `edgequake-query`, and `edgequake-storage`.

### Crate inventory

| Crate | Purpose |
| ----- | ------- |
| `edgequake-api` | HTTP entry point (Axum), WebSocket progress, OpenAPI |
| `edgequake-core` | Orchestration layer and public `EdgeQuake` API |
| `edgequake-pipeline` | Document processing (chunk, extract, embed, merge) |
| `edgequake-query` | RAG query engine (naive/local/global/hybrid modes) |
| `edgequake-storage` | PostgreSQL + pgvector + Apache AGE adapters |
| `edgequake-pdf` | PDF → markdown conversion (vision LLM) |
| `edgequake-tasks` | Background task queue, workers, cancel/fairness |
| `edgequake-auth` | JWT, API keys, OIDC, tenant context |
| `edgequake-audit` | Audit event logging |
| `edgequake-rate-limiter` | Tenant rate limiting (token bucket) |
| `edgequake-observability` | Tracing, metrics, request correlation |

### Dependency flow

```
edgequake-api
  ├── edgequake-core
  │     ├── edgequake-pipeline → edgequake-pdf
  │     ├── edgequake-query
  │     └── edgequake-storage
  ├── edgequake-tasks
  ├── edgequake-auth
  ├── edgequake-audit
  ├── edgequake-rate-limiter
  └── edgequake-observability
```

LLM provider implementations (OpenAI, Ollama, mock, …) are composed at runtime via `edgequake-core` — not a standalone crate.

See the [Architecture Overview](/docs/architecture/overview/) for the full picture and [Crate Reference (detailed)](/docs/architecture/crates/README/) for per-crate notes.
