---
title: 'Deep Dive: Cost Tracking'
---

# Deep Dive: Cost Tracking

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

> How EdgeQuake tracks and reports LLM costs

LLM operations have real monetary costs. EdgeQuake captures token usage across extraction, gleaning, embedding, and query paths, and exposes summaries via dedicated cost endpoints.

**Base URL:** `http://localhost:8080/api/v1`

---

## Overview

```
Document ingest → CostTracker (pipeline) → workspace cost store
Query / chat    → per-request LLM metrics → /costs/* aggregation
```

Costs are recorded per operation type (`extract`, `glean`, `summarize`, `embed`, `query`) and surfaced in ingestion progress (`cost_usd` on `IngestionProgressResponse`) and workspace dashboards.

---

## Cost API endpoints

| Method | Path | Purpose |
| ------ | ---- | ------- |
| `GET` | `/api/v1/pipeline/costs/pricing` | Model pricing table (per 1K tokens) |
| `POST` | `/api/v1/pipeline/costs/estimate` | Estimate cost for hypothetical token usage |
| `GET` | `/api/v1/costs/summary` | Workspace cost summary |
| `GET` | `/api/v1/costs/history` | Cost history over time |
| `GET` | `/api/v1/costs/budget` | Budget status |
| `PATCH` | `/api/v1/costs/budget` | Update budget limits |

```bash
# Pricing configuration
curl "http://localhost:8080/api/v1/pipeline/costs/pricing" \
  -H "X-Workspace-ID: {workspace_id}"

# Workspace summary
curl "http://localhost:8080/api/v1/costs/summary" \
  -H "X-Workspace-ID: {workspace_id}"
```

> **Removed:** `/api/v1/rag/upload` — uploads use `/api/v1/documents/upload` or `/api/v1/documents/pdf`.

---

## Upload paths (where costs originate)

| Upload type | Endpoint | Response |
| ----------- | -------- | -------- |
| Text / file (multipart) | `POST /api/v1/documents/upload` | `202 Accepted` + `task_id` for async ingest |
| PDF (vision convert + ingest) | `POST /api/v1/documents/pdf` | `PdfUploadResponse` with `task_id` |
| JSON text body | `POST /api/v1/documents` | Document record; may enqueue ingest |

Monitor cost accumulation via `GET /api/v1/ingestion/{task_id}/progress` (`cost_usd` field) or WebSocket `ChunkProgress` events (`cost_usd`, `tokens_in`, `tokens_out`).

```bash
# PDF upload — use returned task_id for progress/cost polling
curl -X POST "http://localhost:8080/api/v1/documents/pdf" \
  -H "X-Workspace-ID: {workspace_id}" \
  -F "file=@document.pdf"

# Poll ingest progress (includes cost_usd when available)
curl "http://localhost:8080/api/v1/ingestion/{task_id}/progress" \
  -H "X-Workspace-ID: {workspace_id}"
```

---

## Core data structures (pipeline crate)

### ModelPricing

```rust
pub struct ModelPricing {
    pub model: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
}
```

### CostBreakdown

Per-job summary: `operations` map, `total_input_tokens`, `total_output_tokens`, `total_cost_usd`.

### CostTracker

Thread-safe accumulator used during pipeline execution. Records per-operation token counts and computes USD via `ModelPricing`.

---

## Default model pricing

Built-in pricing covers common OpenAI models (see `/pipeline/costs/pricing` for live values):

| Model | Input / 1K | Output / 1K | Use case |
| ----- | ---------- | ----------- | -------- |
| `gpt-5-nano` | ~$0.00015 | ~$0.0006 | Entity extraction (recommended) |
| `text-embedding-3-small` | ~$0.00002 | N/A | Default embeddings |

Local providers (Ollama, LM Studio) report **$0.00** — useful for dev, slower at scale.

---

## Operation types

| Operation | Typical share | Description |
| --------- | ------------- | ----------- |
| `extract` | 60–70% | Entity/relationship extraction |
| `glean` | 15–25% | Multi-pass refinement |
| `summarize` | 5–10% | Community summaries |
| `embed` | 5–10% | Vector generation |
| `query` | per-query | RAG answer generation |

---

## Cost optimization

1. **Model selection** — `gpt-5-nano` for extraction; reserve larger models for complex queries.
2. **Hybrid providers** — OpenAI LLM + Ollama embeddings (`EDGEQUAKE_EMBEDDING_PROVIDER=ollama`).
3. **Gleaning passes** — reduce `max_gleaning_iterations` for cost-sensitive workloads.
4. **Monitor workspace summary** — `GET /costs/summary` and `/costs/history` for trends.

Typical ingest cost with `gpt-5-nano` (order of magnitude):

| Document size | Approx. cost |
| ------------- | ------------ |
| 10 KB | ~$0.003 |
| 100 KB | ~$0.015 |
| 1 MB | ~$0.10 |

---

## Query costs

Each query mode incurs LLM + optional rerank cost. Hybrid mode is the default balance. Query costs roll into workspace `/costs/*` aggregates.

---

## See Also

- [Pipeline Progress](/docs/deep-dives/pipeline-progress/) — `cost_usd` in progress payloads
- [REST API](/docs/api-reference/rest-api/) — cost endpoint pointers
- [Configuration](/docs/operations/configuration/) — provider and model settings
