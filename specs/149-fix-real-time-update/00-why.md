# 00 — Why (Five Whys)

## Symptom

On `https://demo.edgequake.com/documents` (API v0.26.5, auth on):

- Banner: **Connection Lost. Real-time updates are unavailable.**
- Toast: **Unable to reconnect. Real-time updates unavailable.**
- Manual Refresh still works via HTTP; push path is dead.

## Five Whys

1. **Why is the banner shown?**  
   `wsMaxReconnectsReached` after `ProgressWebSocket` exhausts reconnect attempts.

2. **Why do reconnects fail forever?**  
   The singleton captures `?token=` once at first `getWebSocketClient()`. On login page that is often empty; login/`setTokens` never rebuilds the client URL.

3. **Why would a successful socket still miss document progress?**  
   FE sends `{type:"subscribe", track_ids:[...]}`; BE ignores unknown commands and `event_visible_to_session` returns `false` for every track-scoped event when the session has a workspace claim (auth on).

4. **Why did tests not catch this?**  
   `e2e_websocket.rs` accepts 101/400/426 without exchanging frames; Playwright WS observation is disabled; automated browsers skip auto-connect.

5. **Why does PDF detail still look "live" sometimes?**  
   Polling + SSE fallback. SSE uses credential-less `EventSource` → 401 when auth is enabled, so only polling remains — masking the protocol break until reconnects fail loudly.

## Product cost

Operators lose stage/page feedback during long PDF/entity extraction; they assume the pipeline is stuck; support burden rises; SPEC-083 isolation work unintentionally broke the only multiplexed FE transport.

## Success condition

Authenticated user opens Documents after login → socket connects with current token → subscribe to active tracks → `StageTransition` / `PdfPageProgress` update the row without a second list GET → disconnect recovers or Retry rebuilds credentials cleanly.
