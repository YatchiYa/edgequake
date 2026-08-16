# 02 — Cross-ref matrix

| Ref | Role for SPEC-132 |
|-----|-------------------|
| [#378](https://github.com/raphaelmansuy/edgequake/issues/378) | Symptom: multi-PDF stuck / not uploaded |
| [#361](https://github.com/raphaelmansuy/edgequake/issues/361) / [#365](https://github.com/raphaelmansuy/edgequake/issues/365) | Capacity / slow — **out of fix scope**; vocabulary sibling |
| [#350](https://github.com/raphaelmansuy/edgequake/issues/350) | WebUI N× single-file admits (MD e2e only) |
| [#236](https://github.com/raphaelmansuy/edgequake/issues/236) | Batch API OpenAPI / `/pdf/batch` |
| [#300](https://github.com/raphaelmansuy/edgequake/issues/300) | Progress key collision (client batch id) — mitigated SPEC-054 |
| SPEC-014 | `/documents/pdf/batch` multi-file multipart |
| SPEC-054 | Progress identity prefers server `task_id` |
| SPEC-057 | Wake channel = wake signal; claim_next is truth |
| SPEC-091 F-091-19 / LD-12 | HTTP must not hang on wake `send().await` |
| SPEC-098 / GH-350 | Dropzone multi-file + `spec350` (text) |
| SPEC-122 | Bulk latency honesty / measurement |
| SPEC-123 | `/upload/batch` rejects PDFs |

```ascii
  #378 Plane A ──► LAW-132-1/2/3 ──► admit honesty
  #361 Plane B ──► SPEC-122 only ──► capacity FAQ / UX
         │
         ▼
  Shared progress SSOT: SPEC-054 task_id
```

## Doc ↔ code anchors

| Concern | Path |
|---------|------|
| Dropzone multi | `edgequake_webui/src/hooks/use-document-dropzone.ts` |
| Cap-3 executor | `edgequake_webui/src/lib/upload/bounded-file-upload.ts` |
| PDF upload route | `edgequake_webui/src/lib/upload/perform-file-upload.ts` |
| Batch orchestration | `edgequake_webui/src/hooks/use-file-upload.ts` |
| XHR timeout | `edgequake_webui/src/lib/upload/upload-timeout.ts` |
| Single PDF admit | `edgequake-api/.../pdf_upload/upload.rs` |
| PDF batch admit | same module `upload_pdf_batch_document` |
| Text batch PDF reject | `edgequake-api/.../upload/batch_upload.rs` |
| Enqueue SSOT | `edgequake-api/src/state/task_runtime.rs` |
| Wake channel | `edgequake-tasks/src/queue.rs` |
| Budget caps | `edgequake-core/src/resource/budget.rs` |
| Progress identity | `edgequake-api/.../pdf_upload/progress_identity.rs` |

## Cross-refs

- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
