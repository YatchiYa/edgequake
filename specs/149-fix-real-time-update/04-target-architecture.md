# 04 — Target Architecture

## Principles

- **SRP:** URL factory, transport, subscription set, normalizer, UI status are separate modules.
- **DRY:** One `event_track_id` on server; one normalizer on client; one auth header builder for REST/SSE.
- **Open/closed:** New `ProgressEvent` variants extend filter helper + normalizer switch; handlers stay thin.

## Connection ownership

```ascii
  AuthStore / tokens
        │
        ▼
  resolveWebSocketUrl()   ← every connect()
        │
        ▼
  ProgressWebSocket
    - desiredSubs: Set<track_id>
    - generation: u64
    - backoff capped
        │
        ▼
  wss://origin/ws/pipeline/progress?token=...
```

Session changes (login, logout, refresh): `resetWebSocketClient()` → new URL → reconnect → replay `desiredSubs`.

## Multiplexed protocol

```ascii
  Client                          Server session
  ──────                          ──────────────
  subscribe [t1,t2]  ──────────►  validate each via get_task_for_context
                                  add owned ids to HashSet
                     ◄──────────  SubscribedAck { accepted, rejected_count? opaque }
  broadcast StageTransition(t1)
                     ◄──────────  if t1 in set → send
  unsubscribe [t1]   ──────────►  remove
```

Foreign track: treated as rejected without distinguishing "not found" vs "forbidden" in ACK payload (LAW-149-5). Optional: omit rejected ids entirely; only echo accepted.

## Event filter (DRY)

```rust
fn event_track_id(event: &ProgressEvent) -> Option<&str>;
fn should_forward(event, session, subscribed: &HashSet<String>) -> bool {
  // heartbeats / Connected / Message → true
  // bulk deletion with workspace → workspace match
  // track-scoped → subscribed.contains(track)  // ownership already enforced at subscribe
  // global Job* → session.workspace_id.is_none() only
}
```

Per-track endpoint `/ws/progress/{id}` remains: ownership at upgrade; filter by `matches_track_id` (reuse `event_track_id`).

## Wire normalizer

```ascii
  raw JSON  →  normalizeProgressEvent()  →  WebSocketProgressMessage
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
   ingestion store  docs cache   pdf/chunk hooks
```

Map `pdf_id→document_id`, `page_num→current_page`, derive `progress`, unwrap `{type,data}`.

## SSE

```ascii
  usePdfProgress
      │
      ▼
  streamPdfProgress(trackId, { signal })
      │
      ▼
  streamClient / event-aware fetch  (+ Authorization via buildHeaders)
```

## Non-goals in architecture

- Sticky sessions / Redis pubsub for multi-replica (call out in honest assessment).
- JWT workspace claim rotation for every workspace switch (HTTP headers remain primary for REST; WS uses token claims + owned-task gate — subscribe already scopes by task ownership across workspaces the user can see via task storage).
