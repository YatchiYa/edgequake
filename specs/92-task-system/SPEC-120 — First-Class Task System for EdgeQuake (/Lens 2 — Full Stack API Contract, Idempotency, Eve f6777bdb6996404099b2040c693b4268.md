# Lens 2 — Full Stack: API Contract, Idempotency, Event Stream

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). Normative for the wire contract between the Rust API, the SDKs, and the web interface. Defers to Lens 1 for the operation names, Lens 3 for state semantics, Lens 8 for labels.
> 

## Assessing the surface that exists

The branch has grown one endpoint per historical need rather than one resource with verbs.

| Existing surface | Problem |
| --- | --- |
| `POST /api/v1/tasks/{track_id}/cancel` | canonical and correct — keep as the primitive |
| `DELETE /api/v1/documents/pdf/{pdf_id}/cancel` | cancel expressed as `DELETE` on a nested noun; semantics duplicated |
| `POST /api/v1/pipeline/cancel` | cancels "the pipeline" with no explicit subject |
| `DELETE /api/v2/workspaces/{id}/jobs/{job_id}` | a third cancel spelling, on a `jobs` noun that has no table behind it |
| WebSocket `{ "type": "cancel", "track_id": … }` | a fourth cancel path, unauthenticated at the message level, no acknowledgement |
| `GET /api/v1/tasks/{id}/status` versus `GET /api/v1/tasks/{id}` | two shapes for one resource |

Four spellings of one verb is a DRY violation at the contract layer, and it is why `ingestion_status_mapper.rs` has to reconcile vocabularies at read time. The web client shows the consequence: `use-ingestion-progress.ts` cancels through the socket (`wsCancel(trackId)`) and gets no receipt, so the interface must guess.

## Designing one resource, four verbs

```
/v1/operations                     the only task noun
│
├─ POST   /v1/operations                     create (Idempotency-Key required)
├─ GET    /v1/operations?filter=…            list (keyset cursor, never offset)
├─ GET    /v1/operations/{id}                read one, ETag
├─ POST   /v1/operations/{id}/cancel         request cancel (If-Match optional)
├─ POST   /v1/operations/{id}/retry          re-queue from failed / dead_letter
├─ GET    /v1/operations/{id}/events         replayable stream, ?after_seq=
└─ GET    /v1/operations/{id}/children       pipeline graph (convert → insert)

Legacy paths remain as thin aliases that translate and delegate to the above,
so exactly one implementation of each verb exists (DRY), and the SDKs in
sdks/{rust,typescript,python,go,java,php,ruby,swift} generate from one schema.
```

### Cancel is accepted, not performed

```
POST /v1/operations/{id}/cancel

202 Accepted
{
  "id": "insert-…",
  "state": "cancelling",           ← stored, not derived
  "cancel_requested_at": "2026-07-27T00:58:12Z",
  "cancellable_until": null,       ← non-null only for destructive ops before fence
  "expected_stop_by": "2026-07-27T00:58:17Z"
}

409 Conflict   already terminal, body carries the terminal state and reason
404 Not Found  unknown id, or not visible to this tenant (never leak existence)
423 Locked     destructive operation already fenced — cannot be cancelled
```

Returning `202` with `cancelling` rather than `200` with `cancelled` is the whole point: the response now tells the truth documented in hub gap G1, and `expected_stop_by` gives the interface a deadline to render against instead of a spinner with no horizon.

### Idempotency and concurrency

| Concern | Rule |
| --- | --- |
| Duplicate create | `Idempotency-Key` header, unique per `(tenant, key)`; replay returns the original operation and `200`, never a second job |
| Duplicate cancel | naturally idempotent: setting `cancel_requested_at` when already set is a no-op returning the same body |
| Lost update on retry | `If-Match` on the operation `ETag`, derived from `(state, updated_at)` |
| Racing delete and reprocess | both create jobs; the fence epoch orders them, and the loser gets `409` with `fence_epoch` in the body |

## Streaming state instead of polling it

```
CLIENT                          API                      POSTGRES
  │                              │                          │
  │  GET /events?after_seq=41    │                          │
  │ ────────────────────────► │  SELECT … seq > 41       │
  │                              │ ──────────────────────► │  append-only events
  │ ◄─ 42 progress …            │ ◄────────────────────── │
  │ ◄─ 43 state=cancelling      │  LISTEN task_events      │
  │ ◄─ 44 state=cancelled       │                          │
  │                              │                          │
  reconnect → resume at last seq; no gaps, no duplicates beyond at-least-once
```

The event log is the same table Lens 3 defines for audit. One writer, two readers: the socket bridge (`pipeline_ws_bridge.rs`) and the history endpoint. That removes the current split where progress arrives over the socket while status arrives over HTTP and the two can disagree.

### Event schema

```json
{
  "seq": 44,                      // monotonic per operation
  "operation_id": "insert-…",
  "job_id": "job-…",
  "kind": "state_changed",        // state_changed | progress | warning | fence
  "at": "2026-07-27T00:58:16Z",
  "state": "cancelled",
  "cancel_requested_at": "2026-07-27T00:58:12Z",
  "progress": { "stage": "embedding", "done": 128, "total": 512 },
  "error": null
}
```

The client never derives a state name; it renders `state` plus `cancel_requested_at` through the single mapping table in Lens 8. `IngestionStatusMapper` shrinks from a reconciliation engine to a pure presentation function.

## Handling the optimistic path honestly

```
USER CLICKS STOP
  │
  ├─ optimistic: badge → "Stopping…", stop button disabled, spinner on badge
  │
  ├─ 202 received → keep "Stopping…", start countdown to expected_stop_by
  │
  ├─ event state=cancelled → badge "Cancelled", offer "Reprocess"
  │
  ├─ expected_stop_by passed with no event
  │     → badge "Stopping… (taking longer than usual)", never silently revert
  │
  └─ 409 already terminal → reconcile to the returned state, no error toast
```

Rolling the optimistic state back on a slow cancel is the failure mode to avoid; it teaches users that the stop button does not work even when it does.

## Error taxonomy

| Code | Meaning | Client behaviour |
| --- | --- | --- |
| `operation_not_cancellable` | terminal already | reconcile silently |
| `operation_fenced` | destructive step committed | explain, hide stop |
| `tenant_at_capacity` | admission or fairness refusal | show queue position, do not treat as failure |
| `payload_too_large` | byte admission refusal (`InFlightByteBudget`) | actionable size guidance |
| `provider_unavailable` | retryable upstream failure | show retry countdown from `available_at` |
| `dead_letter` | attempts exhausted | offer explicit retry, surface last error |

The distinction that matters commercially: `tenant_at_capacity` is not an error, it is a queue. Presenting it as an error is what makes fair queueing feel like breakage.

## Where to read next

Operation names come from Lens 1. `state` values and `cancel_requested_at` semantics come from Lens 3. Cursor and filter performance rules come from Lens 4. Timeout and heartbeat budgets that set `expected_stop_by` come from Lens 5. Progress granularity comes from Lens 7. Every label in this document is defined once in Lens 8.