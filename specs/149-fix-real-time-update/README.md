# SPEC-149 — Fix Real-Time Document Updates

> **Mission:** Restore authenticated real-time progress on Documents (and related
> surfaces). The demo banner "Connection Lost / Real-time updates unavailable"
> is a product-visible failure of the WebSocket lifecycle and protocol contract,
> not a transient network blip.
>
> **Method:** First-principles SSOT for (1) auth-aware connection ownership,
> (2) secure multiplexed track subscriptions, (3) one wire normalizer, (4)
> authenticated SSE via fetch streaming — with unfakable E2E proofs.
>
> **Preserves:** SPEC-083 fail-closed tenant isolation. Track events never fan
> out to unauthorized or unsubscribed sessions.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  LAW: Real-time is a contract — connect with current credentials, subscribe  │
│  only to owned tracks, deliver typed events, recover without lying.          │
│                                                                              │
│  Root causes (stacked):                                                      │
│    1. Singleton WS URL freezes pre-login (tokenless) forever                 │
│    2. FE sends subscribe; BE ignores it and drops ALL track events on /ws    │
│    3. Wire DTO drift (PdfPageProgress fields, ProgressSnapshot, etc.)        │
│    4. SSE EventSource cannot send Authorization → 401 in auth mode           │
│                                                                              │
│  Target:                                                                     │
│    Auth-gated connect + rebuild on login/refresh                             │
│    Typed subscribe/unsubscribe with get_task_for_context gate                │
│    One progress-event-normalizer.ts                                          │
│    Authenticated fetch SSE (stream-client)                                   │
│    make spec149-proof                                                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Document map

```ascii
  README
    → 00-why (5 WHY)
    → 01-first-principles (LAW-149-*)
    → 02-cross-ref-matrix
    → 03-root-cause
    → 04-target-architecture
    → 05-lenses/ (PO, fullstack, security, UX, test reliability)
    → 06-ux-ui-spec
    → 07-implementation-plan
    → 08-e2e-test-matrix
    → 09-edge-cases
    → 10-acceptance
    → 11-honest-assessment
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| I1 | Auth-aware WS lifecycle + subscription replay | Done |
| I2 | Backend typed subscribe + track filter | Done |
| I3 | Wire normalizer + GraphStorage/ProgressSnapshot | Done |
| I4 | Authenticated SSE via stream-client | Done |
| T1 | Rust unit + e2e_spec149_realtime_progress | Done |
| T2 | Vitest production client + normalizer | Done |
| T3 | Playwright no-polling frame E2E | Done |
| A1 | Acceptance | Done |

## Locked decisions

| Decision | Choice |
|----------|--------|
| Transport | Keep one tab-global `/ws/pipeline/progress` socket |
| Isolation | Subscribe set + `get_task_for_context` (SPEC-083) |
| Per-track WS | Keep `/ws/progress/{track_id}` for compatibility |
| Auth on browser WS | `?token=` query (browsers cannot set Authorization) |
| SSE | Authenticated `fetch` stream; retire native EventSource |
| Wire schema | Rust tagged `{type,data}` is SSOT; FE normalizes once |
| Multi-replica bus | Out of scope; document as known limitation |

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-048](../048-document-progress/) | Progress event family |
| [SPEC-083](../083-security-hardening/) | WS origin/token, tenant filter |
| [SPEC-086](../086-unified-stage/) | StageTransition |
| [SPEC-027](../027-api-contract/) | Auth / JWT claims |
| GitHub #277 | CORS + WS `?token=` |

## Non-goals

- Distributed progress bus (Redis / LISTEN-NOTIFY)
- Changing Helm ingress defaults beyond documenting `/ws` requirement
- Replacing polling safety-net entirely
- Cookie-only WS auth migration
