# 08 — E2E Test Matrix

| ID | Scenario | Layer | Assert |
|----|----------|-------|--------|
| U-149-01 | Connect uses latest token after setTokens | Vitest | URL contains token; old URL abandoned |
| U-149-02 | No connect while auth pending | Vitest | connect not called without token when auth on |
| U-149-03 | Reconnect replays desiredSubs | Vitest | subscribe frames resent on open |
| U-149-04 | CONNECTING guard + retry reset | Vitest | single socket; attempts=0 after reset |
| U-149-10 | PdfPageProgress normalizer | Vitest | page_num → current_page; progress derived |
| U-149-11 | StageTransition passthrough | Vitest | data envelope preserved |
| U-149-20 | PDF SSE uses Authorization | Vitest | fetch called with bearer header |
| C-149-01 | Owner subscribe receives StageTransition | Rust e2e | frame type+task_id |
| C-149-02 | Foreign workspace subscribe silent | Rust e2e | no track event |
| C-149-03 | Unsubscribe stops delivery | Rust e2e | no further frames for track |
| C-149-04 | Missing token → 401 | Rust e2e | not 101 |
| C-149-05 | Bad Origin → 403 | Rust e2e | not 101 |
| C-149-06 | Duplicate subscribe idempotent | Rust e2e | still one delivery path |
| E-149-01 | Login then WS has token | Playwright | websocket URL includes token |
| E-149-02 | Pushed StageTransition updates row | Playwright | DOM changes; documents GET blocked after load |
| E-149-03 | Reconnect replays subscribe | Playwright | framesent subscribe after close/reopen |
| E-149-04 | Banner clears after Retry | Playwright | banner hidden |

## Commands

```bash
make spec149-proof
cargo test -p edgequake-api --test e2e_spec149_realtime_progress
cd edgequake_webui && pnpm exec vitest run src/lib/websocket/__tests__ src/lib/api/__tests__/stream-client
PLAYWRIGHT_SKIP_STACK_CHECK=1 pnpm exec playwright test e2e/spec149-real-time-update.spec.ts --project=chromium
```

## Cross-refs

- Edges: [09-edge-cases.md](09-edge-cases.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
