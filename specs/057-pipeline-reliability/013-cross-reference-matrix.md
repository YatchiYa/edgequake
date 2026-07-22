# 013 — Cross-Reference Matrix

**Spec:** SPEC-057

---

## REQ ↔ code ↔ env ↔ test

| REQ | Primary symbols | Env | Test / proof |
| --- | --------------- | --- | ------------ |
| REQ-057-01 | `ChannelTaskQueue`, `task_runtime`, future claim loop | `EDGEQUAKE_STARTUP_AUTO_RESUME` | restart hydrate tests; new claim tests |
| REQ-057-02 | `CancellationRegistry`, `apply_task_row_cancel`, worker dequeue guards | — | `contract_cancel_and_fairness`; restart-after-cancel (new) |
| REQ-057-03 | `PdfProcessingStatus`, `task_impl` PDF Failed mapping | — | PDF cancel status assertion (new) |
| REQ-057-04 | `IngestionStatusMapper`, `stage_bridge`, `status_updates` legacy helpers | — | `ingestion_status_mapper` fixture matrix; `contract_ingestion_status_mapper` |
| REQ-057-05 | `display_status` / `ui_phase=stopping`, `cancelTask` UI, StatusBadge Stopping… | — | `e2e/spec057-cancel-status-ssot.spec.ts`; `contract_cancel_and_fairness` |
| REQ-057-06 | `IngestionFailureClass::Cancelled`, `Task::can_retry` | — | taxonomy unit tests in `ingestion_reliability.rs` |
| REQ-057-07 | `process_pdf_processing` → `process_text_insert` | `TASK_PROCESSING_TIMEOUT_SECS` | phase-split integration (new) |
| REQ-057-08 | LargeDocumentProfile / SPEC-038 timeout helpers | `EDGEQUAKE_PDF_PARSER_BACKEND` | SPEC-038 battle / e2e |
| REQ-057-09 | `resolve_worker_pool_limits`, `TenantConcurrencyLimiter` | `MAX_TASKS_PER_TENANT`, `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY`, `EDGEQUAKE_LLM_PROVIDER` | hybrid clamp test (new) |
| REQ-057-10 | `validate_delivery_for_replicas`, `claim_next`, Bridged/NotifyOnly wake | `EDGEQUAKE_REPLICAS`, `EDGEQUAKE_TASK_DELIVERY` | `contract_multi_replica_claim`; `contract_spec026_task_delivery` |
| REQ-057-11 | partial `MergeArtifacts`, KV `compensation_quarantine:*`, `compensation_quarantine_total` | — | `edgequake-storage` compensate tests; `contract_compensate_observability` |
| REQ-057-12 | `store_contention` assessor → queue-metrics + `/ready` | `EDGEQUAKE_DB_POOL_UTIL_*`, `EDGEQUAKE_COMPENSATION_QUARANTINE_*` | `contract_compensate_observability`; runbook in `docs/ingestion-cancel-and-fairness.md` |
| REQ-057-13 | `classify_ingestion_failure` | — | `spec045_ingestion_reliability` |
| REQ-057-14 | slim `ProcessingResult`, `ensure_embeddings` | — | `contract_spec047_ingest_p5_reliability` |
| REQ-057-15 | cancel/fairness/restart contracts | — | `contract_cancel_and_fairness` + extensions |

---

## CAUSE ↔ prior SPEC

| CAUSE | Prior SPEC | Relationship |
| ----- | ---------- | ------------ |
| 01, 05 | SPEC-054 | Startup auto-resume / boot semantics |
| 02, 03, 10 | docs/ingestion-cancel-and-fairness; SPEC-048/050 | Cancel UX + status |
| 04, 11 | SPEC-038 | Large PDF / timeout / Vision |
| 06 | cancel/fairness ops note | Local clamp caveat |
| 07, 12 | SPEC-045 | Saga + failure taxonomy |
| 08 | SPEC-047 P5 | Slim checkpoints |
| 09 | SPEC-026 delivery | Bridged/NotifyOnly |
| 11 | SPEC-010/011 | Historical embed/reliability |
| track_id UX | SPEC-056 | Progress identity |

---

## Ops doc ↔ API ↔ UI

| Ops note | API | UI call site |
| -------- | --- | ------------ |
| Canonical cancel | `POST /api/v1/tasks/{track_id}/cancel` | `pipeline.ts` `cancelTask`; `use-document-mutations` |
| Job DELETE alias | `DELETE /api/v2/workspaces/{id}/jobs/{job_id}` | jobs UI (if present) |
| PDF cancel alias | `DELETE /api/v1/documents/pdf/{pdf_id}/cancel` | `use-pdf-progress.ts` |
| Cancel all | `POST /api/v1/pipeline/cancel` | `pipeline-stages-card` / pipeline dialog |
| WS cancel | `{type:cancel, track_id}` | WS client hooks |
| Fairness metrics | `GET /api/v1/pipeline/queue-metrics` | admin/ops (partial) |
| Restart policy | `EDGEQUAKE_STARTUP_AUTO_RESUME` | Reprocess CTAs (SPEC-051) |

---

## Lens ↔ CAUSE coverage

| Lens | Primary CAUSEs |
| ---- | -------------- |
| PO 004 | 01, 03, 05, 06, 11, 12 |
| UX 005 | 02, 03, 05, 10 |
| UI 006 | 03, 10 |
| Full Stack 007 | 01, 02, 04, 09, 10 |
| O(n) 008 | 04, 08, 11 |
| Postgres 009 | 01, 02, 05, 09 |
| AGE/pgvector 010 | 07, 12 |
| AI 011 | 06, 08, 11, 12 |

---

## Document graph

```text
  000-index
     │
     ├── 001-five-whys ──────────────► 012-causes
     ├── 002-first-principles
     ├── 003-code-is-law ────────────► 013-xref
     ├── 004..011 lenses ────────────► 012 + 014
     ├── 012-causes ◄────────────────► 014-plan
     └── 013-xref
```

Next: [014-improvement-plan.md](./014-improvement-plan.md)
