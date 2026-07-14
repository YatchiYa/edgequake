# SPEC-050 — SRE Lens

## Reliability Considerations

### Delete Failure Modes

```
┌────────────────────────────────────────────────────────────────────┐
│  Failure                  │  Current behavior    │  Target         │
├────────────────────────────────────────────────────────────────────┤
│  Graph cascade error      │  500 returned        │  Partial success│
│  (AGE Cypher timeout)     │                      │  + WS broadcast │
│                           │                      │  + UI shows ⚠   │
├────────────────────────────────────────────────────────────────────┤
│  Vector delete error      │  Warning logged,     │  Same (non-fatal│
│                           │  continue            │  + shown in UI) │
├────────────────────────────────────────────────────────────────────┤
│  KV delete error          │  Returns error       │  Retry once,    │
│                           │                      │  then surface   │
├────────────────────────────────────────────────────────────────────┤
│  Task cancel failure      │  Continues delete    │  Log + proceed  │
│  (task already done)      │                      │  (non-fatal)    │
└────────────────────────────────────────────────────────────────────┘
```

### Observability: Metrics to Add

```rust
// In delete handler, emit structured tracing spans:
// span: "document.delete.impact"       — duration of impact analysis
// span: "document.delete.cancel_task"  — duration of in-flight task cancellation
// span: "document.delete.vectors"      — duration of vector deletion
// span: "document.delete.graph"        — duration of graph cascade
// span: "document.delete.kv"           — duration of KV deletion

// Counter: document.delete.success{workspace_id, partial}
// Counter: document.delete.error{workspace_id, phase, error_type}
// Histogram: document.delete.duration_ms{workspace_id}
```

### Blast Radius Analysis

| Scenario                        | Blast radius                           | Mitigation                                                     |
| ------------------------------- | -------------------------------------- | -------------------------------------------------------------- |
| Bulk delete with 1000 docs      | Full KG cleared for workspace          | Require "DELETE ALL" confirmation + X-EdgeQuake-Confirm header |
| Delete while ingestion running  | In-flight pipeline writes after delete | Task cancel before KV removal (already implemented)            |
| Delete fails mid-way            | Orphaned vectors / orphaned KV         | Non-fatal partial, reconcile job (future)                      |
| Network drop during bulk delete | Partial state on client                | WS reconnect broadcasts final state                            |

### Health Check Integration

The `/health` endpoint should expose aggregate deletion stats for SRE monitoring:

```json
{
  "status": "healthy",
  "recent_deletions": {
    "last_24h_count": 12,
    "last_24h_partial_failures": 0
  }
}
```

### Alert Thresholds

| Metric                          | Warning | Critical |
| ------------------------------- | ------- | -------- |
| Delete duration >               | 5s      | 30s      |
| Delete partial_failure rate >   | 5%      | 20%      |
| Bulk delete stuck (no progress) | 2 min   | 5 min    |
| WS broadcast lag >              | 500ms   | 2s       |

## Deployment Safety

- The new WS events (`DeletionStarted`, `DeletionCompleted`, etc.) are additive — no breaking change.
- Frontend consumers ignore unknown event types (already implemented in WS hook).
- The `DeleteDocumentResponse` shape is extended with optional fields — no breaking change.
- `DeletionImpactCard` can degrade gracefully if `GET /deletion-impact` fails (shows "Impact analysis unavailable" and still allows confirm).
