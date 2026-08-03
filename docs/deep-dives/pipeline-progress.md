---
title: 'Deep Dive: Pipeline Progress Tracking'
---

# Deep Dive: Pipeline Progress Tracking

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

> Real-time monitoring of document ingestion, PDF conversion, and deletion.

EdgeQuake exposes progress through **WebSocket streams**, **REST polling**, and **SSE**. All examples use `http://localhost:8080` (default backend port).

**Authoritative contract:** [`edgequake_webui/openapi/openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) (version `0.23.0`).

---

## Progress identity (SPEC-054)

Every async operation has a server-issued **`task_id`** (also called `track_id` in progress stores). This is the sole key for progress, cancel, retry, and WebSocket subscription.

| Field | Source | Use |
| ----- | ------ | --- |
| `task_id` | Upload / PDF / reprocess response | **Subscribe here** |
| `track_id` (optional) | Client batch correlation | Echo only — **not** a progress key |

PDF upload returns `PdfUploadResponse.task_id` (format `pdf-<uuid>`). Text/file upload returns `FileUploadResponse.task_id` when async.

---

## Endpoints at a glance

| Channel | Path | Scope |
| ------- | ---- | ----- |
| WebSocket (global) | `ws://localhost:8080/ws/pipeline/progress` | All pipeline events in workspace context |
| WebSocket (filtered) | `ws://localhost:8080/ws/progress/{track_id}` | Single upload track (PDF page progress, snapshots) |
| REST (ingestion) | `GET /api/v1/ingestion/{track_id}/progress` | Full ingest stage breakdown (SPEC-048) |
| REST (batch) | `POST /api/v1/ingestion/progress` | Multiple tracks in one call |
| REST (PDF poll) | `GET /api/v1/documents/pdf/progress/{track_id}` | PDF phase progress (6 phases) |
| SSE (PDF stream) | `GET /api/v1/documents/pdf/progress/stream/{track_id}` | Push PDF progress until complete/fail |
| REST (task) | `GET /api/v1/tasks/{track_id}` | Task row status + metadata |
| Cancel | `POST /api/v1/tasks/{track_id}/cancel` | Canonical cancel (see [ingestion cancel doc](../ingestion-cancel-and-fairness.md)) |

> **Removed:** `/api/v1/rag/progress/*` — do not use. All progress moved to the paths above.

---

## Convert → ingest (SPEC-057 P2)

PDF admission is a **two-task pipeline**:

```
┌───────────────────────────────────────────────────────┐
│ Convert then ingest (SPEC-057)                        │
│                                                       │
│  POST /documents/pdf  -->  admit task_id              │
│              |                                        │
│              v                                        │
│  [1] PdfProcessing (convert only)                     │
│      vision / edgeparse --> markdown                  │
│      PDF row --> Completed (artifact)                 │
│              |                                        │
│              v  markdown barrier                      │
│  [2] Insert (KG ingest, new lease)                    │
│      chunk --> extract --> embed --> store            │
│              |                                        │
│              v                                        │
│  document display_status = completed                  │
└───────────────────────────────────────────────────────┘
```

```
┌─────────────────────────────────────────────────────┐
│ Progress channels (port 8080)                       │
│                                                     │
│  WS  /ws/pipeline/progress      (global)            │
│  WS  /ws/progress/{track_id}    (filtered)          │
│  GET /ingestion/{id}/progress                       │
│  GET /documents/pdf/progress/{id}                   │
│  SSE /documents/pdf/progress/stream/{id}            │
│  POST /tasks/{id}/cancel        (canonical)         │
└─────────────────────────────────────────────────────┘
```

| Phase | Task type | Timeout key | PDF row on success |
| ----- | --------- | ----------- | ------------------ |
| Convert | `pdf_processing` | `LargeDocumentProfile::convert_timeout_secs` | `Completed` + markdown |
| Ingest | `insert` | `LargeDocumentProfile::ingest_timeout_secs` | unchanged |

**UI implication:** PDF `Completed` means convert finished, **not** full ingestion. Doc `current_stage` may still be `extracting` while PDF is `Completed`. `IngestionStatusMapper` keeps showing the doc stage (see below).

Cancel of convert **or** PDF cancel cancels both linked Pending/Processing tasks for the same `pdf_id`.

---

## Document status: `display_status` / `ui_phase` (SPEC-057 P4)

List and detail responses include presentation fields from `IngestionStatusMapper`. **Prefer these over re-deriving from raw `status`.**

| Field | Values | Meaning |
| ----- | ------ | ------- |
| `display_status` | `uploading`, `converting`, `extracting`, `embedding`, `storing`, `completed`, `indexed`, `failed`, `cancelled`, … | Badge key |
| `ui_phase` | `idle` \| `running` \| `stopping` \| `terminal` | Lifecycle phase for spinners |

### Stopping → Cancelled

When the user cancels:

1. `POST /api/v1/tasks/{track_id}/cancel` sets cancel intent in `CancellationRegistry`.
2. While cooperative shutdown runs: `ui_phase=stopping`, `display_status` may still show the active stage (e.g. `extracting`). UI shows **"Stopping…"**.
3. When task/doc/PDF reach terminal cancel truth: `display_status=cancelled`, `ui_phase=terminal`.

Cancel is cooperative — expect a short delay until the current LLM/vision round-trip aborts.

---

## WebSocket: global pipeline stream

`ws://localhost:8080/ws/pipeline/progress`

Receives workspace-scoped `ProgressEvent` JSON (`type` + `data`):

| Event | When |
| ----- | ---- |
| `Connected` | Handshake complete |
| `StatusSnapshot` | Initial state on connect |
| `JobStarted` | Batch job begins |
| `DocumentProgress` | Per-document counts |
| `ChunkProgress` | Chunk-level ingest (SPEC-048 DEF-02) |
| `GraphStorageProgress` | Merge/store sub-phases |
| `PdfPageProgress` | PDF page extraction |
| `ChunkFailure` | Recoverable chunk error |
| `DocumentFailed` | Document terminal failure |
| `BatchCompleted` / `JobFinished` | Job completion |
| `DeletionStarted` / `DeletionPhase` / `DeletionCompleted` | Single-doc delete (SPEC-050) |
| `BulkDeletionStarted` / `BulkDeletionItemProgress` / `BulkDeletionCompleted` | Bulk delete |
| `Heartbeat` | Keepalive (~30s) |
| `CancellationRequested` | Cancel acknowledged |

```javascript
const ws = new WebSocket('ws://localhost:8080/ws/pipeline/progress');
ws.onmessage = (event) => {
  const { type, data } = JSON.parse(event.data);
  if (type === 'ChunkProgress') {
    console.log(`Chunk ${data.chunk_index + 1}/${data.total_chunks}`);
  }
};
```

Clients may send `{ "type": "cancel", "track_id": "..." }` — equivalent to `POST /tasks/{track_id}/cancel`.

---

## WebSocket: per-track filter

`ws://localhost:8080/ws/progress/{track_id}`

Streams only events for the given `track_id` (typically the upload `task_id`). Ideal for single PDF upload pages.

Messages: `Connected`, `StatusSnapshot`, `PdfPageProgress`, `Heartbeat`.

```javascript
const taskId = uploadResponse.task_id; // NOT optional client track_id
const ws = new WebSocket(`ws://localhost:8080/ws/progress/${taskId}`);
ws.onmessage = (event) => {
  const { type, data } = JSON.parse(event.data);
  if (type === 'PdfPageProgress') {
    console.log(`Page ${data.page_num}/${data.total_pages}: ${data.phase}`);
  }
};
```

---

## REST: ingestion progress

### Single track

```bash
curl -H "X-Workspace-ID: {workspace_id}" \
     -H "Authorization: Bearer {token}" \
     "http://localhost:8080/api/v1/ingestion/{track_id}/progress"
```

Returns `IngestionProgressResponse`: `track_id`, `document_id`, `stage`, `stage_status`, nested `progress` (stages array, `completion_percentage`, `eta_seconds`), optional `counts` (`pages` \| `chunks` \| `entities` \| `relationships`), optional `cost_usd`.

### Batch

```bash
curl -X POST "http://localhost:8080/api/v1/ingestion/progress" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: {workspace_id}" \
  -d '{"track_ids": ["pdf-abc", "insert-def"]}'
```

---

## REST / SSE: PDF progress

### Poll

```bash
curl "http://localhost:8080/api/v1/documents/pdf/progress/{track_id}" \
  -H "X-Workspace-ID: {workspace_id}"
```

Returns `PdfUploadProgress`: `phases[]`, `overall_percentage`, `is_complete`, `is_failed`.

Use **`PdfUploadResponse.task_id`** as `{track_id}` — not the optional client batch `track_id`.

### SSE stream

```bash
curl -N "http://localhost:8080/api/v1/documents/pdf/progress/stream/{track_id}" \
  -H "X-Workspace-ID: {workspace_id}"
```

Stream closes when `is_complete` or `is_failed`, client disconnects, or no progress for 60 seconds.

```javascript
const es = new EventSource(
  `/api/v1/documents/pdf/progress/stream/${taskId}`,
  { withCredentials: true }
);
es.onmessage = (e) => {
  const p = JSON.parse(e.data);
  console.log(`${p.overall_percentage}%`);
  if (p.is_complete || p.is_failed) es.close();
};
```

---

## Deletion progress (SPEC-050)

Document delete broadcasts phase-granular events on **`/ws/pipeline/progress`** (same global socket):

| Phase | Label |
| ----- | ----- |
| `cancelling_task` | Cancelling in-flight ingestion (if processing) |
| `removing_vectors` | pgvector embeddings |
| `removing_graph` | AGE entities/edges cascade |
| `removing_kv` | Chunks, content, metadata |
| `finalizing` | Content-hash cleanup |

Event sequence: `DeletionStarted` → `DeletionPhase` (repeat) → `DeletionCompleted`.

Preview impact before delete: `GET /api/v1/documents/{document_id}/deletion-impact`.

Delete response (`200`) includes `chunks_deleted`, `entities_affected`, `relationships_affected`, `embeddings_deleted`, `partial_failure` — not `204`.

Bulk delete (`DELETE /api/v1/documents` or workspace clear) emits `BulkDeletionStarted` → `BulkDeletionItemProgress` → `BulkDeletionCompleted`.

---

## Recommended client patterns

| Scenario | Pattern |
| -------- | ------- |
| Documents list badges | Poll list/detail; read `display_status` + `ui_phase` |
| Single PDF upload page | `ws://…/ws/progress/{task_id}` or PDF SSE |
| Pipeline dashboard | `ws://…/ws/pipeline/progress` |
| Background poller | `GET /ingestion/{track_id}/progress` or `GET /tasks/{track_id}` |
| Cancel button | `POST /tasks/{track_id}/cancel`; show Stopping until `ui_phase=terminal` |

---

## Observability

`GET /api/v1/pipeline/queue-metrics` — tenant park waiters, cancel intent counts, store contention SLOs. See [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md).

---

## See Also

- [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md) — cancel SSOT, fairness, restart
- [Cost Tracking](/docs/deep-dives/cost-tracking/) — token/cost in progress payloads
- [REST API](/docs/api-reference/rest-api/) — guided API overlay
- [OpenAPI snapshot](../../edgequake_webui/openapi/openapi.snapshot.json) — full schema
