# 04 — Target architecture

## Admit path (target)

```ascii
  Handler
    │
    ├─1─ Persist task (DB) ────────────── durable truth
    │
    ├─2─ Wake try_send / send_timeout ─── never hang HTTP
    │      │
    │      ├─ Ok  → worker woken
    │      └─ Full/timeout → log + rely on hydrate/claim
    │
    └─3─ 202 + task_id + queue_position ── Plane A success
```

SSOT: `TaskRuntime::enqueue` / delivery helper — **all** task types (LAW-132-8). PDF helpers keep calling `state.enqueue_task` unchanged at call sites.

## WebUI path (target)

```ascii
  N files → bounded executor (3)
    │
    ├─ per-file XHR with timeout
    ├─ on timeout/error → row = error; slot freed (finally)
    ├─ on 2xx → startTracking(server task_id)
    └─ siblings continue independently
```

Keep N× `/documents/pdf` (LAW-132-9). Do not migrate WebUI to `/pdf/batch` in v1.

## Vocabulary (target)

| UI phrase | Means |
|-----------|--------|
| Transferring / Saving to workspace | Plane A in flight |
| Queued / Waiting for slot | Admitted; capacity (SPEC-122) |
| Converting / Extracting | Plane B |
| Upload failed (per file) | Plane A failed/timed out |
| Never: “Upload stuck” for healthy queued convert | LAW-132-6 |

## Non-goals in architecture

- Raising vision concurrency unboundedly
- Dual client stacks (batch + N×)
- Changing CHECK status allowlist

## Cross-refs

- As-is: [03-code-as-is.md](03-code-as-is.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
