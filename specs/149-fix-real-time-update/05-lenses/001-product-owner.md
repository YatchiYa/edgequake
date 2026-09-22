# 05 — Lenses

## 001 — Product Owner

- **Outcome:** Documents page shows live stage/page progress after login without Refresh.
- **Visible failure today:** Red Connection Lost banner + toast on demo.
- **Acceptance language:** User never needs Reload to restore realtime after login/token refresh.
- **Risk if skipped:** Support tickets, false "pipeline stuck" reports, distrust of progress UI.

## 002 — Fullstack

- Single multiplexed socket scales better than per-track sockets for list views.
- Server must own subscription state; client desired-set is advisory until ACK.
- Normalizer prevents N divergent parsers (store, cache, hooks).
- SSE must share REST auth path — EventSource is a dead end under JWT.

## 003 — Security

- Preserve Origin allow-list and token gate (SPEC-083 / #277).
- Subscribe validation via `get_task_for_context` — no cross-workspace event leak.
- ACK must not enumerate foreign track existence.
- Query-token remains necessary for browser WS; log redaction of `token=` is ops concern (existing).
- Do not weaken CORS fail-closed to "fix" connectivity.

## 004 — UX / UI

- Banner only after true max reconnects; Retry must work.
- One stable toast id for disconnect/max; dismiss on restore.
- Connected state should clear banner without full navigation.
- Polling remains safety net; UI must not flash stage backward (LAW-149-12).

## 005 — Test Reliability

- No OR-list handshake statuses.
- Playwright: register `page.on('websocket')` **before** navigation/connect; assert frames.
- Vitest must import production modules (no reimplemented helpers).
- Automated browsers must exercise the same connect path as humans (or explicit connect in test).
- `make spec149-proof` = focused unfakable gate for CI.
