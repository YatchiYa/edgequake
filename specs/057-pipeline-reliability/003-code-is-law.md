# 003 — Code Is Law (Evidence Map)

**Spec:** SPEC-057  
**Rule:** Every reliability claim cites a live symbol. If code disagrees with narrative, code wins.

---

## Cancel fan-in (ASCII)

```text
  POST /api/v1/tasks/{track_id}/cancel
  DELETE .../jobs/{job_id}
  DELETE .../pdf/{pdf_id}/cancel
  POST /api/v1/pipeline/cancel
  WS { type: cancel, track_id }
            │
            ▼
  ┌─────────────────────────────────────┐
  │ services::task_cancel               │
  │   apply_task_row_cancel             │
  │   apply_cancel_all_active           │
  └──────────────┬──────────────────────┘
                 │
       ┌─────────┴──────────┐
       ▼                    ▼
  CancellationRegistry   TaskStorage
  cancel intent+token    mark_cancelled()
       │
       ▼
  Worker dequeue skips intent / Cancelled
  Pipeline select! / token checks at .await
```

---

## Evidence table

| Claim | File | Symbol / evidence | Gap |
| ----- | ---- | ----------------- | --- |
| Delivery SSOT is Postgres claim (P1) | `edgequake-tasks/src/storage.rs` + `postgres.rs` | `claim_next` / `refresh_lease` / `release_claim` + mig 088 | Channel is wake-only |
| Channel wake (ephemeral) | `edgequake-tasks/src/queue.rs` | `ChannelTaskQueue` — bounded `mpsc` | Missed wake recovered by claim poll |
| Horizontal delivery hooks exist | `edgequake-api/src/state/task_runtime.rs` | `delivery_mode_from_env`, `BridgedTaskQueue`, `NotifyOnly` | Multi-replica Bridged still P3 default |
| Enqueue goes through delivery helper | `edgequake-tasks/src/delivery/mod.rs` | `enqueue_with_delivery` | Wake after durable INSERT |
| Worker claim loop | `edgequake-tasks/src/worker.rs` | wake/poll → `claim_next` → fairness `release_claim` | — |
| Boot: Pending survives | `edgequake/src/main.rs` | `recover_orphaned_tasks` — Processing only | AUTO_RESUME ≠ discard Pending |
| Cancel SSOT for task row + registry | `edgequake-api/src/services/task_cancel.rs` | `apply_task_row_cancel` | Doc KV update stays in handlers (by design) |
| Cancel intents are process-local | `edgequake-tasks/src/cancellation.rs` | `cancel_intents: RwLock<HashSet<String>>` | Lost on process death |
| Fairness parks (no requeue storm) | `edgequake-tasks/src/tenant_limiter.rs` | `TenantConcurrencyLimiter` + `acquire()` park | — |
| Worker timeout / lease heartbeat / retry | `edgequake-tasks/src/worker.rs` | `refresh_lease` CAS + `timeout(process)` | Whole-task timeout wraps PDF+KG (P2) |
| Permanent failure taxonomy | `edgequake-tasks/src/ingestion_reliability.rs` | `IngestionFailureClass`, `classify_ingestion_failure` | Novel strings → `Unknown` |
| PDF+KG coupled in one task | `edgequake-api/src/processor/pdf_processing.rs` | after convert → `process_text_insert(...)` | Long lease; one timeout |
| PDF cancel → Cancelled (SPEC-057 P0) | `handlers/pdf_upload/operations.rs`, `services/ingestion_status.rs` | `pdf_status_for_cancel()` | — |
| PDF status includes Cancelled | `edgequake-storage/src/pdf_storage.rs` | `PdfProcessingStatus::Cancelled` + mig 087 | Full stage_bridge mapper still deferred (P0.3b) |
| HTTP cancel sets failure_class | `handlers/tasks.rs` + `apply_doc_cancelled_fields` | `failure_class=cancelled` | — |
| Vision cancel string classified | `ingestion_reliability.rs` | `is_cancel_failure_message` / `"cancelled during"` | — |
| Persist order + compensate | `edgequake-pipeline/src/persistence/ingestion_persister.rs` | KV → vector → merge; `compensate_merge_failure` | Crash window |
| Slim checkpoint re-embed | `edgequake-api/src/processor/text_insert/extraction.rs` | comment SPEC-047 P5; `ensure_embeddings` | Extra embed cost on resume |
| Queue metrics expose park/cancel | `edgequake-api/src/handlers/pipeline.rs` | `tenant_park_waiters`, `max_tasks_per_tenant` | — |
| Local fairness clamp from env | `.env.example` + `main.rs` `resolve_worker_pool_limits` | local provider → 1/tenant unless allow flag | May diverge from runtime extract model |
| Startup auto-resume opt-in | `.env.example` | `EDGEQUAKE_STARTUP_AUTO_RESUME` default off | Orphans need Reprocess |
| UI cancel API | `edgequake_webui/src/lib/api/edgequake/pipeline.ts` | `cancelTask` → `POST /tasks/${taskId}/cancel` | Also PDF cancel path in `use-pdf-progress.ts` |
| UI cancel from documents | `edgequake_webui/src/hooks/use-document-mutations.ts` | `cancelMutation` → `cancelTask(trackId)` | Stopping… copy varies by surface |

---

## Status model fragmentation (law)

```text
  Layer              Terminal set
  ─────────────────  ──────────────────────────────────────
  TaskStatus         Indexed | Failed | Cancelled
  Doc KV (API)       completed/indexed | failed | cancelled | partial_failure
  PdfProcessingStatus Completed | Failed | Cancelled  ← SPEC-057 P0
  Core DocumentStatus Processed | Failed          ← library path, poorer
  Unified stages     Completed | Failed (+ cancelled via bridge strings)
```

---

## What is already solid (do not regress)

1. **Per-task cancel registry** replaced global boolean (`cancellation.rs` WHY block).  
2. **DRY cancel apply** in `task_cancel.rs` for HTTP/WS/PDF/pipeline.  
3. **Tenant park-not-churn** fairness.  
4. **SPEC-045 taxonomy** wired into workers (`Cancelled` permanent).  
5. **Checkpoints / extraction snapshots** for resume without full re-LLM (when hydrate/reprocess paths hit them).  
6. **Compensation hook** on merge failure.

---

## Industry north-star (external)

**P1 done:** claim with `FOR UPDATE SKIP LOCKED` + leases matches Postgres-native durable queues. Remaining: Convert/Ingest split (P2), multi-replica Bridged default (P3).

Next: lenses [004](./004-product-owner-lens.md) → [011](./011-ai-engineer-lens.md)
