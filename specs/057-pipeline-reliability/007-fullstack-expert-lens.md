# 007 — Full Stack Expert Lens

**Spec:** SPEC-057  
**Key question:** Where are the SOLID seams, and what DRY consolidations unlock reliability?

---

## Scope

API contracts, worker/runtime wiring, FE/BE cancel matrix, delivery abstraction. Out of scope: AGE Cypher internals (lens 010).

---

## Cancel API matrix

| Entry | Handler area | Shared apply | Doc KV update |
| ----- | ------------ | ------------ | ------------- |
| `POST /tasks/{id}/cancel` | `handlers/tasks.rs` | `apply_task_row_cancel` | yes (handler) |
| `DELETE .../jobs/{id}` | workspaces jobs | same | yes |
| `DELETE .../pdf/{id}/cancel` | pdf_upload | find PdfProcessing → same | yes |
| `POST /pipeline/cancel` | `handlers/pipeline.rs` | `apply_cancel_all_active` | bulk |
| WS `{type:cancel}` | `handlers/websocket.rs` | same | yes |
| UI `cancelTask` | `pipeline.ts` | → task cancel | — |
| UI PDF cancel | `use-pdf-progress.ts` | → PDF cancel alias | — |

**DRY win already shipped:** `services/task_cancel.rs`. Remaining DRY debt: status projection + PDF enum.

---

## SOLID assessment

| Principle | Current | Gap |
| --------- | ------- | --- |
| **S**RP | `task_cancel` owns row+registry; persister owns saga | Status mapping scattered |
| **O**CP | `TaskQueue` + delivery modes | Worker still assumes channel semantics for resume |
| **L**SP | Bridged/NotifyOnly implement `TaskQueue` | Resume/hydrate paths special-cased |
| **I**SP | `TaskProcessor` narrow | PdfProcessing does convert+ingest (fat) |
| **D**IP | Worker → traits | Boot wiring in `main.rs` / `task_runtime.rs` OK |

---

## Delivery seam (target architecture)

```text
  TODAY                         TARGET (P1)
  ─────                         ───────────
  admit → INSERT task           admit → INSERT task (Pending)
       → channel.send                → NOTIFY / channel wake
  worker ← channel.recv         worker ← claim SKIP LOCKED
       → process                     → process + heartbeat lease
  restart: lose channel         restart: re-claim Pending/stale Processing
```

Keep `ChannelTaskQueue` as **wake accelerator** behind `TaskQueue`; claim loop is SSOT (REQ-057-01, 10).

---

## Stage bridge debt

`stage_bridge.rs` documents deferred collapse across Tasks / Unified / Internal layers → CAUSE-057-10. Full-stack fix: one `IngestionStatusView` DTO for API+WS+UI.

---

## Recommendations → REQ

| Engineering move | SOLID/DRY | REQ |
| ---------------- | --------- | --- |
| `IngestionStatusMapper` SSOT | SRP + DRY | REQ-057-04 |
| `PdfProcessingStatus::Cancelled` | ISP of status domain | REQ-057-03 |
| DB claim loop + channel wake | OCP/DIP on `TaskQueue` | REQ-057-01, 10 |
| Durable cancel via task status | DRY with existing Cancelled | REQ-057-02 |
| Split ConvertTask / IngestTask | SRP / ISP | REQ-057-07 |
| Contract tests expand restart | proof | REQ-057-15 |

**Out of scope:** Rewriting OpenAPI surface cosmetics; authZ matrix for cancel.

Next: [008-on-expert-lens.md](./008-on-expert-lens.md)
