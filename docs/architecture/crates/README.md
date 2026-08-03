---
title: "Architecture: Crate Reference"
sidebar:
  hidden: true
---

# Architecture: Crate Reference

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../../ingestion-cancel-and-fairness.md)

> EdgeQuake's modular Rust workspace (11 crates)

EdgeQuake splits responsibilities across focused crates with clear dependency boundaries. **Code is law:** the list below matches `edgequake/crates/` on disk.

---

## Crate inventory

| # | Crate | Role |
| - | ----- | ---- |
| 1 | `edgequake-api` | Axum HTTP/WS server, handlers, middleware |
| 2 | `edgequake-core` | Orchestrator, config, LLM provider factory |
| 3 | `edgequake-pipeline` | Ingestion pipeline, prompts, progress/cost tracking |
| 4 | `edgequake-query` | Multi-mode RAG retrieval and generation |
| 5 | `edgequake-storage` | KV, vector (pgvector), graph (AGE) traits + Postgres |
| 6 | `edgequake-pdf` | PDF extraction, vision convert, markdown output |
| 7 | `edgequake-tasks` | Task rows, worker pool, cancel registry, fairness |
| 8 | `edgequake-auth` | Authentication, authorization, multi-tenancy helpers |
| 9 | `edgequake-audit` | Structured audit events |
| 10 | `edgequake-rate-limiter` | Per-tenant API rate limits |
| 11 | `edgequake-observability` | Tracing subscriber, Prometheus metrics, correlation |

**Not separate crates:** `edgequake-llm` and `edgequake-graph` do not exist. LLM traits/providers are wired through `edgequake-core` / `edgequake-pipeline` / `edgequake-query`. Graph storage lives in `edgequake-storage` (AGE backend).

---

## Dependency graph

```
┌─────────────────────────────────────────────────────────────────┐
│                    EDGEQUAKE CRATE HIERARCHY (v0.23.0)          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                      ┌──────────────────┐                       │
│                      │  edgequake-api   │  ◀── HTTP + WebSocket  │
│                      └────────┬─────────┘                       │
│           ┌───────────────────┼───────────────────┐             │
│           ▼                   ▼                   ▼             │
│  ┌────────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │edgequake-tasks │  │edgequake-core│  │edgequake-auth    │    │
│  └────────────────┘  └──────┬───────┘  └──────────────────┘    │
│                               │                                 │
│              ┌────────────────┼────────────────┐                │
│              ▼                ▼                ▼                │
│    ┌─────────────────┐ ┌────────────┐ ┌─────────────┐          │
│    │edgequake-pipeline│ │edgequake-  │ │edgequake-   │          │
│    │                  │ │   query    │ │  storage    │          │
│    └────────┬─────────┘ └────────────┘ └─────────────┘          │
│             ▼                                                   │
│    ┌─────────────────┐                                          │
│    │  edgequake-pdf  │                                          │
│    └─────────────────┘                                          │
│                                                                 │
│  Cross-cutting: edgequake-audit, edgequake-rate-limiter,        │
│                 edgequake-observability                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core crates

### edgequake-core

Orchestration layer — `EdgeQuake` public API, config, provider selection, document/query coordination.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-core` |
| Depends on | pipeline, query, storage |

Key types: `EdgeQuake`, `EdgeQuakeConfig`, `Orchestrator`.

---

### edgequake-api

Axum server exposing `/api/v1/*`, Ollama-compat `/api/*`, WebSocket progress, MCP, health probes.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-api` |
| Framework | Axum |

Key modules: `handlers`, `routes`, `middleware`, `services::ingestion_status_mapper`.

Notable routes: `/ws/pipeline/progress`, `/ws/progress/{track_id}`, `/api/v1/documents/*`, `/api/v1/ingestion/{track_id}/progress`.

OpenAPI: `/swagger-ui/`, snapshot at `edgequake_webui/openapi/openapi.snapshot.json`.

---

### edgequake-pipeline

Document processing — chunking, entity/relationship extraction, gleaning, embedding, graph merge, progress and cost trackers.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-pipeline` |

Sub-modules: `chunker`, `extractor`, `prompts`, `progress`, `stage_bridge`.

---

### edgequake-query

RAG query engine with strategies: `Naive`, `Local`, `Global`, `Hybrid`, `Mix`.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-query` |

---

### edgequake-storage

Storage traits and PostgreSQL implementations (pgvector + Apache AGE). In-memory adapters exist for tests only — production requires `DATABASE_URL`.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-storage` |

Key traits: `VectorStorage`, `GraphStorage`, `KvStorage`.

---

### edgequake-pdf

PDF → markdown via embedded pdfium + vision LLM. Used by the convert phase before KG ingest.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-pdf` |

---

## Infrastructure crates

### edgequake-tasks

Postgres-backed task queue SSOT: `claim_next`, lease heartbeat, tenant fairness, cooperative cancel (`CancellationRegistry`). Powers async upload, PDF convert/insert, reprocess, rebuild jobs.

| Attribute | Value |
| --------- | ----- |
| Path | `edgequake/crates/edgequake-tasks` |

Task types include `PdfProcessing` (convert) and `Insert` (ingest) — see [Pipeline Progress](/docs/deep-dives/pipeline-progress/).

---

### edgequake-auth

JWT login/refresh, API keys, OIDC, tenant/workspace context extraction.

| Path | `edgequake/crates/edgequake-auth` |

---

### edgequake-audit

Append-only audit event sink for compliance logging.

| Path | `edgequake/crates/edgequake-audit` |

---

### edgequake-rate-limiter

Token-bucket rate limiting applied per tenant in API middleware.

| Path | `edgequake/crates/edgequake-rate-limiter` |

---

### edgequake-observability

Single init point for `tracing`, Prometheus metrics (`/metrics`), request IDs, OTEL hooks.

| Path | `edgequake/crates/edgequake-observability` |

---

## Feature flags (selected)

| Flag | Crate | Description |
| ---- | ----- | ----------- |
| `postgres` | storage | PostgreSQL backends |
| `pdf` | pipeline | PDF ingest integration |
| `otel` | observability | OpenTelemetry export |
| `metrics` | observability | Prometheus recorder |

Provider selection is **runtime** via environment (`EDGEQUAKE_LLM_PROVIDER`, `OPENAI_API_KEY`, …) — not a separate LLM crate feature flag.

---

## See Also

- [Architecture Overview](/docs/architecture/overview/) — high-level design
- [Data Flow](/docs/architecture/data-flow/) — ingest and query paths
- [REST API](/docs/api-reference/rest-api/) — HTTP surface
- [Pipeline Progress](/docs/deep-dives/pipeline-progress/) — task/progress model
