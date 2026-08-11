# 06 — UX / UI Spec

## Happy path

```ascii
  User clicks Delete
       → Document status: Deleting (progress if available)
       → Cleanup discovers edges (indexed)
       → Cascade + post-proof
       → Status: Deleted / absent from list
```

## Failure path (discovery timeout)

| Layer | Behavior |
|-------|----------|
| Storage | Returns `StorageError::Database` with singular-edge timeout message |
| API worker | Marks task failed; persists reason |
| API mapping (v1 stretch) | If message contains `statement timeout` / `Source-prefix singular`, rewrite to product copy |
| UI | Show mapped title/body; Retry |

## Detection helper (DRY SSOT)

[`graph_cleanup_timeout.rs`](../../edgequake/crates/edgequake-api/src/services/graph_cleanup_timeout.rs):

- `is_source_discovery_timeout` (wraps `is_db_statement_timeout` + singular probe strings)
- `graph_cleanup_timeout_user_message(Delete|Reprocess)` — no raw Postgres
- `log_graph_cleanup_timeout` — detail in logs only
- Delete worker + `retract_document_indexes_checked` (reprocess) both use this SSOT

## Copy (normative)

**Delete title/body:** Graph cleanup timed out. Retry delete…

**Reprocess:** Graph cleanup timed out during reprocess. Retry reprocess…

Raw `statement timeout` / `Source-prefix…` strings must not appear in the primary user message.
