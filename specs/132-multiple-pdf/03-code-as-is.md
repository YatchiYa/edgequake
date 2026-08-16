# 03 — Code as-is

## WebUI (Documents)

```ascii
  document-dropzone (multiple=true default)
       │
       ▼
  use-file-upload.handleFilesUpload
       │  shared batchTrackId (correlation)
       │  Promise.all + createBoundedExecutor(3)
       ▼
  performFileUpload
       ├─ pdf  → uploadPdfDocument → POST /documents/pdf
       ├─ image→ POST /documents/upload
       └─ text → POST /documents (JSON)
```

| Fact | Location |
|------|----------|
| Cap 3 | `MAX_CONCURRENT_FILE_UPLOADS = 3` in `bounded-file-upload.ts` |
| XHR timeout | `60s + 8s/MiB`, max 600s in `upload-timeout.ts` |
| Vision default | `enable_vision: true` on PDF path in `perform-file-upload.ts` |
| Progress key | Prefer server `task_id` (`progress-track-id.ts`) |
| E2E multi | `e2e/spec350-bulk-upload-webui.spec.ts` — **MD only** |
| API batch e2e | `e2e/issue-236-batch-upload-api.spec.ts` — `/pdf/batch` |

## API routes

| Endpoint | Multi-file | PDF |
|----------|------------|-----|
| `POST /documents/pdf` | No (single) | Yes — WebUI path |
| `POST /documents/pdf/batch` | Yes | Yes — SDK/API |
| `POST /documents/upload/batch` | Yes | **Rejected** (SPEC-123) |

## Admit sequence (single PDF)

```ascii
  multipart stream → validate ≤50MiB → BYTEA store
       → create PdfProcessing task (Pending)
       → state.enqueue_task  ──► persist + channel.send().await
       → 202 + task_id + queue_position (best-effort)
```

**Gap:** `ChannelTaskQueue::send` uses blocking `send().await` when capacity (100) is full — HTTP handler waits. SPEC-091 QW2 documented non-block; current `e2e_spec091_queue_admission` only asserts ETA projection, not wake non-block.

## Post-admit (not #378 fix scope)

```ascii
  Worker claim_next
       → PdfProcessing (vision jobs ≤2)
       → Insert (extract/embed)
       → searchable
```

Docker defaults: `WORKER_THREADS=8`, `MAX_TASKS_PER_TENANT=6`.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
