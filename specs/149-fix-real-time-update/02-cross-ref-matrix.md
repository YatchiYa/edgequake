# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Browser cannot set WS Authorization | MDN WebSocket constructor; WHATWG WebSockets |
| Production demo auth on + same-origin API | Injected `__EDGEQUAKE_RUNTIME_CONFIG__` on demo.edgequake.com |
| Caddy proxies `/ws*` | `deploy/gcp/compose/snippets.caddy` |
| WS token query + Origin | SPEC-083 / GitHub #277 / `middleware::ws_validate_*` |
| Track ownership | `services/task_scope.rs::get_task_for_context` |
| StageTransition | SPEC-086 |
| Progress event family | SPEC-048 / `websocket_types::ProgressEvent` |

## Code SSOT (as-is → target)

| Concern | As-is | Target |
|---------|-------|--------|
| WS URL | Frozen at singleton create | Resolved each connect |
| Auto-connect | Always (except Playwright detect) | After auth ready |
| Subscribe cmd | Sent by FE, ignored by BE | Typed handler + HashSet |
| Track events on `/ws` | Always filtered out if scoped | Forward if subscribed+owned |
| Pdf fields | `page_num` vs `current_page` | Normalizer once |
| SSE | `EventSource` no auth | `streamClient` + AbortSignal |
| E2E | Handshake status OR-list | Real 101 + frame |

## Requirement map

| REQ | Law | WP | Primary paths | Tests | AC |
|-----|-----|----|---------------|-------|----|
| REQ-149-01 Live credentials | LAW-149-1/2 | WP-149-3 | `websocket-manager.ts`, provider | U-149-01, E-149-01 | AC-149-01 |
| REQ-149-02 Secure subscribe | LAW-149-3/4/5 | WP-149-1/2 | `websocket.rs`, types | C-149-01..04 | AC-149-02 |
| REQ-149-03 Wire normalize | LAW-149-6 | WP-149-4 | `progress-event-normalizer.ts` | U-149-10 | AC-149-03 |
| REQ-149-04 Replay subs | LAW-149-7 | WP-149-3 | `progress-websocket.ts` | U-149-03, E-149-03 | AC-149-04 |
| REQ-149-05 Recovery UX | LAW-149-8/9 | WP-149-3 | provider, banner | U-149-04 | AC-149-05 |
| REQ-149-06 Auth SSE | LAW-149-10 | WP-149-5 | `stream-client.ts`, pdf hook | U-149-20 | AC-149-06 |
| REQ-149-07 Unfakable proof | LAW-149-11 | WP-149-6 | e2e + Playwright | C/E-149-* | AC-149-07 |

## Related specs

| Spec | Relationship |
|------|--------------|
| SPEC-083 | Parent isolation; amended delivery path only |
| SPEC-086 | StageTransition consumer |
| SPEC-048 | Event vocabulary |
| SPEC-050 | Deletion events on same bus |
| SPEC-027 | JWT workspace claim defaults |

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
