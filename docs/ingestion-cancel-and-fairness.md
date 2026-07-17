# Ingestion cancel, fairness, and restart semantics

Operational notes for the task worker pool (P0–P3 remediation).

## Cancel a task (canonical)

```http
POST /api/v1/tasks/{track_id}/cancel
```

Effects:

1. Document KV status → `cancelled`
2. Task row → `Cancelled` (terminal; no auto-retry)
3. `CancellationRegistry` records a **cancel intent** and signals any in-flight `CancellationToken`
4. Pending / fairness-parked copies of the same `track_id` are dropped on dequeue

Also supported (all go through `services::task_cancel::apply_task_row_cancel` / `apply_cancel_all_active`):

| Path | Behavior |
|------|----------|
| `DELETE /api/v2/workspaces/{id}/jobs/{job_id}` | Same as task cancel |
| `DELETE /api/v1/documents/pdf/{pdf_id}/cancel` | Finds active `PdfProcessing` task → per-task cancel |
| `POST /api/v1/pipeline/cancel` | Cancels all **registered** in-flight tasks |
| WebSocket `{ "type": "cancel", "track_id": "..." }` | Same registry + task row update |

UI should call `POST /tasks/{track_id}/cancel` and show “Stopping…” until status is terminal.

Cancel is **cooperative**: vision convert, LLM extract, and embed calls abort via `select!` / token checks at `.await` points. Expect a short delay until the current HTTP round-trip is dropped.

## Tenant fairness (no requeue storm)

When `MAX_TASKS_PER_TENANT` > 0 (default ≈ ¾ of `WORKER_THREADS`):

- Workers `try_acquire` a per-tenant semaphore
- If at capacity, the task **parks** on `acquire()` in a background waiter (no 500ms channel bounce)
- Worker continues serving other tenants’ ready work

Local providers (`ollama` / `lmstudio` via `EDGEQUAKE_LLM_PROVIDER`) clamp to **1** concurrent task per tenant unless `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1`. That clamp uses the **configured LLM provider env**, which can diverge from the model actually used for extraction.

## Observability

`GET /api/v1/pipeline/queue-metrics` includes:

- `tenant_park_waiters` — tasks waiting for a tenant permit
- `cancel_intent_count` / `cancel_intent_total`
- `max_tasks_per_tenant`

## Restart semantics

The in-memory channel queue is not durable. On restart:

- Pending rows may be hydrated when `EDGEQUAKE_STARTUP_AUTO_RESUME=1`
- Cancel intents are process-local; cancelled DB status remains the source of truth after restart
- Prefer explicit Reprocess for interrupted work when auto-resume is off (default)
