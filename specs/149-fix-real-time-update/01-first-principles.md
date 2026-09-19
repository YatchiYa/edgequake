# 01 — First Principles (LAW-149)

## Axioms

| ID | Law | Operational meaning |
|----|-----|---------------------|
| **LAW-149-1** | Credentials are live | Every connect attempt reads current access token; never freeze URL at singleton construction |
| **LAW-149-2** | No anonymous handshake when auth on | Do not open WS until auth hydration + token present (or auth disabled) |
| **LAW-149-3** | One socket per tab | Multiplex tracks on `/ws/pipeline/progress`; no N sockets per document |
| **LAW-149-4** | Subscribe is authorization | Deliver track events only if subscribed **and** `get_task_for_context` succeeds |
| **LAW-149-5** | Fail closed | Foreign/unknown tracks: no event leak; subscribe ACK must not reveal existence |
| **LAW-149-6** | One wire SSOT | Rust `ProgressEvent` tagged envelope is canonical; FE has one normalizer |
| **LAW-149-7** | Idempotent desired set | Client retains subscription set; replay after every reconnect |
| **LAW-149-8** | Connect is idempotent | Guard OPEN **and** CONNECTING; generation token ignores stale callbacks |
| **LAW-149-9** | Honest recovery | Cap backoff; Retry resets attempts + rebuilds URL; dismiss infinite toast on restore |
| **LAW-149-10** | Auth on every stream | SSE/fetch streams use same auth header builder as REST; no bare EventSource |
| **LAW-149-11** | Proof is frame exchange | Tests that do not open a socket and assert a typed frame are not proofs |
| **LAW-149-12** | Monotonic UI | Late poll must not roll a document stage backward past a newer WS event |

## Anti-patterns

| Anti-pattern | Violates |
|--------------|----------|
| Capture `url` once in singleton ctor | LAW-149-1 |
| Auto-connect on `/login` without token | LAW-149-2 |
| Drop all track events on global bus "for security" without subscribe | LAW-149-4 |
| Silent ignore of `subscribe` commands | LAW-149-4 |
| Duplicate field mapping in store + cache + hooks | LAW-149-6 |
| `new EventSource(url)` for protected routes | LAW-149-10 |
| Accept 400/426 as WS "success" | LAW-149-11 |

## Env / ops notes

| Concern         | Contract                                                                      |
| -----------------| -------------------------------------------------------------------------------|
| Browser WS auth | `?token=` (MDN: no custom headers on `WebSocket`)                             |
| Origin          | Exact CORS allow-list (SPEC-083); demo uses same-origin Caddy `/ws*`          |
| Next rewrites   | Dev-only `/ws` proxy; production relies on reverse proxy or absolute `apiUrl` |

## Cross-refs

- Root cause: [03-root-cause.md](03-root-cause.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
