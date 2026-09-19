# 07 — Implementation Plan

## Phase A — Spec pack (this directory)

Done when README + laws + lenses + matrices land.

## Phase B1 — Backend protocol (WP-149-1 / WP-149-2)

1. Add typed `ClientCommand` / `SubscribedAck` in `websocket_types.rs`.
2. Refactor `event_track_id` + `should_forward_pipeline_event`.
3. In `handle_pipeline_socket`: maintain `HashSet<String>` subscriptions; handle subscribe/unsubscribe/ping/cancel.
4. Cap subscription set size; ACK accepted ids only.
5. Keep per-track endpoint; share track-id helper.

## Phase B2 — Frontend lifecycle (WP-149-3)

1. `resolveWebSocketUrl()` on every connect; `resetWebSocketClient()` on session change.
2. Gate connect on auth hydration.
3. Desired subscription set + replay; CONNECTING guard; generation; capped backoff; retry reset.
4. Provider toast hygiene + reconnect helper for banner.

## Phase B3 — Normalizer (WP-149-4)

1. Add `progress-event-normalizer.ts`.
2. Align types; handle GraphStorageProgress + ProgressSnapshot.
3. Route provider/store/cache through normalizer.

## Phase B4 — SSE (WP-149-5)

1. Extend stream-client for named SSE events + AbortSignal.
2. Single `streamPdfProgress` factory; delete EventSource duplicates.
3. Wire `usePdfProgress`.

## Phase B5 — Docs / AsyncAPI

Update `openapi_asyncapi.rs` examples for subscribe/ACK.

## Phase C — Proof (WP-149-6)

1. `e2e_spec149_realtime_progress.rs` + tokio-tungstenite dev-dep.
2. Vitest suites under `src/lib/websocket/__tests__/`.
3. Playwright `e2e/spec149-real-time-update.spec.ts`.
4. `make spec149-proof`.

## Order

```ascii
  A docs → B1 backend → B2 client → B3 normalizer → B4 SSE → B5 asyncapi → C proof
              │              │            │
              └─ unit ───────┴────────────┘
```
