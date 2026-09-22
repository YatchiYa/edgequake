# Lens 004 — UX / UI

## Surfaces

| Surface | Behavior |
|---------|----------|
| ConnectionBanner | Show iff `wsMaxReconnectsReached`; Retry → reset + connect |
| Toast unableToReconnect | Stable id; `duration: Infinity` until restore/dismiss |
| Toast connectionLost | Short-lived; do not stack on every transient drop |
| Toast connectionRestored | Once when recovering from max-reconnects |
| Status badge | Updates from StageTransition / PdfPageProgress via cache patch |

## Copy (keep existing i18n keys)

- `websocket.unableToReconnect` / `unableToReconnectDesc`
- `websocket.connectionRestored` / `connectionRestoredDesc`

## Interaction

Retry on banner and toast both call the same `reconnectRealtime()` helper (DRY).
