# 06 — UX / UI Spec

## Documents header

When realtime is exhausted:

1. Show `ConnectionBanner` with Retry.
2. Show persistent error toast (stable id `ws-max-reconnects`).
3. Keep table readable; Refresh still works.

When Retry or auto-recovery succeeds:

1. Clear banner state (`wsMaxReconnectsReached=false`).
2. Dismiss `ws-max-reconnects` toast.
3. Optional short success toast.

## Progress while connected

- Active rows subscribe via `useDocumentWebSocket`.
- Stage badge / message updates from patched React Query cache on `StageTransition`.
- PDF converting phase shows page counts from normalized `PdfPageProgress`.

## Login flow

- No WS attempt until authenticated (auth on) or auth disabled.
- After login navigation, first documents visit establishes socket with token.

## Accessibility

- Banner is assertive live region (existing markup).
- Retry is a button with clear name.
