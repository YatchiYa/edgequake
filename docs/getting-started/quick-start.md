---
title: 'Quick Start Guide'
---

> **Product: v0.23.0** · Contract: [OpenAPI snapshot](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

# Quick Start Guide

> From zero to your first knowledge graph query in 10 minutes

---

## What You'll Build

By the end of this guide, you will have:

1. ✅ Ingested a document into EdgeQuake (async pipeline)
2. ✅ Polled until entity extraction completes
3. ✅ Queried the graph using natural language
4. ✅ Visualized the knowledge graph in the WebUI

```
┌─────────────────────────────────────────────────────────────┐
│                      Your First Flow                        │
│                                                             │
│   Document ───▶ POST /documents (202) ───▶ track_id         │
│                      │                         │            │
│                      │    poll / WS / SSE      ▼            │
│                      └──────────────▶ Knowledge Graph       │
│                                                             │
│   Query ───────────────────────────▶ Natural language answer│
└─────────────────────────────────────────────────────────────┘
```

---

## Prerequisites

Ensure EdgeQuake is running:

```bash
curl http://localhost:8080/health
# Expected: JSON containing "status":"healthy"
```

- **API:** port **8080**
- **WebUI:** port **3000** (`make dev`)

If not running, see [Installation Guide](/docs/getting-started/installation/).

### Authentication headers

`make dev` sets `EDGEQUAKE_DEV_MODE=true` (open API — no auth headers needed).

For deployments **without** `EDGEQUAKE_DEV_MODE`, obtain a token or API key first:

```bash
# Login (when bootstrap admin is configured)
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"ChangeMe123!"}' | jq -r .access_token)

AUTH_HEADER="Authorization: Bearer $TOKEN"
# Or: AUTH_HEADER="X-API-Key: $EDGEQUAKE_MASTER_API_KEY"
```

Use `$AUTH_HEADER` on all examples below when auth is enabled. See [Runtime auth hardening](../operations/runtime-auth-hardening.md).

---

## Step 1: Ingest Your First Document

Uploads are **asynchronous**: the API returns **HTTP 202 Accepted** with a `document_id` and `track_id`. Entity counts are **not** returned synchronously.

### Option A: Via REST API

```bash
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -d '{
    "content": "Marie Curie was a Polish-French physicist and chemist who conducted pioneering research on radioactivity. She was the first woman to win a Nobel Prize, and the only person to win Nobel Prizes in two different sciences (Physics in 1903, Chemistry in 1911). Curie discovered two elements: polonium (named after Poland) and radium. She worked at the University of Paris with her husband Pierre Curie. Their daughter, Irène Joliot-Curie, also won a Nobel Prize in Chemistry in 1935.",
    "title": "Marie Curie Biography"
  }')

echo "$RESPONSE" | jq
TRACK_ID=$(echo "$RESPONSE" | jq -r .track_id)
DOC_ID=$(echo "$RESPONSE" | jq -r .document_id)
```

**Expected Response** (202):

```json
{
  "document_id": "doc_abc123",
  "track_id": "f6fa9cad-bbff-4892-a855-3bd7d70da044",
  "status": "processing"
}
```

> There is no `entities_extracted` field on async admit. Poll for completion (Step 1b).

### Option B: Via WebUI

1. Open <http://localhost:3000>
2. Navigate to **Documents** → **Upload**
3. Paste the text above or upload a file
4. Watch progress in the UI (WebSocket/SSE)

---

## Step 1b: Wait for Processing

Poll task status until terminal:

```bash
# Poll until completed or failed
until STATUS=$(curl -s -H "$AUTH_HEADER" \
  "http://localhost:8080/api/v1/tasks/$TRACK_ID" | jq -r .status) && \
  [ "$STATUS" = "completed" ] || [ "$STATUS" = "failed" ] || [ "$STATUS" = "cancelled" ]; do
  echo "Status: $STATUS — waiting..."
  sleep 3
done
echo "Final status: $STATUS"
```

Alternative progress endpoints:

| Method | Endpoint |
| ------ | -------- |
| Poll   | `GET /api/v1/ingestion/{track_id}/progress` |
| Poll   | `GET /api/v1/documents/{document_id}` (check `display_status`) |
| SSE    | `GET /api/v1/documents/pdf/progress/stream/{track_id}` (PDF) |
| WebSocket | `ws://localhost:8080/ws/progress/{track_id}` |

To cancel: `POST /api/v1/tasks/{track_id}/cancel` — see [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md).

---

## Step 2: Explore the Knowledge Graph

Graph entities and relationships live under the **`/api/v1/graph/`** namespace (not `/api/v1/entities`).

### View Extracted Entities

```bash
curl -s -H "$AUTH_HEADER" \
  "http://localhost:8080/api/v1/graph/entities" | jq '.entities[:5]'
```

**Expected entities** (names normalized to UPPERCASE_WITH_UNDERSCORES):

```
MARIE_CURIE, PIERRE_CURIE, RADIUM, POLONIUM, NOBEL_PRIZE, …
```

### View Relationships

```bash
curl -s -H "$AUTH_HEADER" \
  "http://localhost:8080/api/v1/graph/relationships" | jq '.relationships[:5]'
```

**Sample relationship**:

```json
{
  "source": "MARIE_CURIE",
  "target": "RADIUM",
  "keywords": ["discovered"],
  "description": "Marie Curie discovered radium"
}
```

### Graph statistics

```bash
curl -s -H "$AUTH_HEADER" http://localhost:8080/api/v1/graph/stats | jq
```

---

## Step 3: Query the Knowledge Graph

### Simple Query

```bash
curl -s -X POST http://localhost:8080/api/v1/query \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -d '{
    "query": "Who discovered radium and when?",
    "mode": "hybrid"
  }' | jq
```

The response includes a natural-language answer and source references (entities/chunks used).

### Try Different Query Modes

```bash
# Local: entity-focused
curl -s -X POST http://localhost:8080/api/v1/query \
  -H "Content-Type: application/json" -H "$AUTH_HEADER" \
  -d '{"query": "What is radium?", "mode": "local"}' | jq .response

# Global: overview
curl -s -X POST http://localhost:8080/api/v1/query \
  -H "Content-Type: application/json" -H "$AUTH_HEADER" \
  -d '{"query": "Summarize the Curie family achievements", "mode": "global"}' | jq .response

# Naive: vector search only
curl -s -X POST http://localhost:8080/api/v1/query \
  -H "Content-Type: application/json" -H "$AUTH_HEADER" \
  -d '{"query": "Who won Nobel Prizes?", "mode": "naive"}' | jq .response
```

---

## Step 4: Visualize in WebUI

1. Open <http://localhost:3000>
2. Navigate to **Graph** (left sidebar)
3. Explore nodes and edges interactively

**WebUI features**: zoom/pan, click nodes for details, filter by entity type, search.

---

## Step 5: Add More Documents

Each upload is async — save the new `track_id` and poll again:

```bash
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" -H "$AUTH_HEADER" \
  -d '{
    "content": "Albert Einstein developed the theory of relativity while working at the Swiss Patent Office in Bern. He won the Nobel Prize in Physics in 1921 for his explanation of the photoelectric effect. Einstein corresponded with Marie Curie and they became friends. Both attended the famous Solvay Conference in 1911.",
    "title": "Albert Einstein"
  }')
echo "$RESPONSE" | jq .track_id
```

Query across both documents:

```bash
curl -s -X POST http://localhost:8080/api/v1/query \
  -H "Content-Type: application/json" -H "$AUTH_HEADER" \
  -d '{"query": "What connections existed between Einstein and Curie?", "mode": "hybrid"}' | jq .response
```

---

## Understanding What Happened

```
┌─────────────────────────────────────────────────────────────┐
│                    Processing Pipeline (async)                │
│                                                             │
│  1. ADMIT          POST /documents → 202 + track_id         │
│  2. CHUNKING       Worker splits into ~1200-token chunks    │
│  3. EXTRACTION     LLM identifies entities & relationships  │
│  4. EMBEDDING      Vectors for chunks, entities, relations  │
│  5. GRAPH WRITE    Nodes + edges → PostgreSQL (AGE)         │
│  6. DEDUP          Similar entities merged                  │
│                                                             │
│  PDFs: convert (vision) → ingest (Insert task) — two phases │
└─────────────────────────────────────────────────────────────┘
```

Workers claim tasks via Postgres `claim_next` + lease. See [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md) for cancel, fairness, and multi-replica semantics.

---

## Quick Reference: API Endpoints

| Endpoint                              | Method | Purpose                    |
| ------------------------------------- | ------ | -------------------------- |
| `/health`                             | GET    | Check server status (no auth) |
| `/api/v1/documents`                   | POST   | Ingest document (202 async) |
| `/api/v1/documents`                   | GET    | List documents             |
| `/api/v1/tasks/{track_id}`            | GET    | Task status                |
| `/api/v1/tasks/{track_id}/cancel`     | POST   | Cancel task                |
| `/api/v1/ingestion/{track_id}/progress` | GET  | Ingestion progress         |
| `/api/v1/query`                       | POST   | Query knowledge graph      |
| `/api/v1/graph/entities`              | GET    | List entities              |
| `/api/v1/graph/relationships`       | GET    | List relationships         |
| `/api/v1/graph/stats`                 | GET    | Graph statistics           |

Contract: [OpenAPI snapshot](../../edgequake_webui/openapi/openapi.snapshot.json).

---

## Next Steps

1. **[Document Ingestion Deep Dive](/docs/tutorials/document-ingestion/)** — Pipeline details
2. **[Architecture Overview](/docs/architecture/overview/)** — System design
3. **[Query Modes](/docs/deep-dives/query-modes/)** — Choosing the right mode
4. **[Runtime auth hardening](../operations/runtime-auth-hardening.md)** — Production auth

---

## Troubleshooting

### No entities after upload

```bash
# Check task finished
curl -s -H "$AUTH_HEADER" "http://localhost:8080/api/v1/tasks/$TRACK_ID" | jq

# Check LLM config
curl -s http://localhost:8080/api/v1/config/effective | jq '.llm'
ollama list   # if using Ollama
```

### 401 Unauthorized

Auth is on by default outside `EDGEQUAKE_DEV_MODE`. Login or set `X-API-Key` (see [Authentication headers](#authentication-headers)).

### Slow processing / bulk upload

Local Ollama is limited to **1 concurrent ingest task per tenant** by default (`MAX_TASKS_PER_TENANT=1`) — bulk completion is intentionally near-serial. Docker defaults are wider (tenant **6**); cloud/`make` with API keys wider still — but wall clock remains LLM + (for PDF) vision bound. **Upload finished ≠ searchable.** See FAQ [Why do bulk uploads feel excessively slow?](../faq.md#why-do-bulk-uploads-feel-excessively-slow-spec-122--361--365) and SPEC-122. Check `GET /api/v1/pipeline/queue-metrics`.

### Empty query results

```bash
curl -s -H "$AUTH_HEADER" http://localhost:8080/api/v1/documents | jq '.documents | length'
curl -s -H "$AUTH_HEADER" http://localhost:8080/api/v1/graph/stats | jq
```
