# 11 — Honest Assessment

## What this fixes

- Demo-visible Connection Lost from tokenless singleton.
- Silent progress blackout from ignored subscribe + blanket track filter.
- PDF SSE 401 under auth.
- Tests that claimed WS worked without reading frames.
- Provider listener orphaning when reconnect destroyed the singleton (UI stuck Offline after a successful handshake).

## What remains limited

| Limitation | Why |
|------------|-----|
| Process-local `broadcast` channel | Multi-replica API pods can miss events; need sticky routing or distributed bus (future spec) |
| JWT default workspace claim | Login tokens pin default workspace; HTTP `X-Workspace-ID` + membership bind for REST; WS scopes by **task ownership** at subscribe — switching workspace still requires client to subscribe to that workspace's track ids (already true for document list) |
| Query-string token | Browser constraint; prefer short-lived access tokens; ticket for first-message auth later |
| Polling safety-net | Intentionally retained for lag/miss; not removed |
| Workspace-wide `cargo clippy -D warnings` | Package still has pre-existing dead_code/needless_return noise outside SPEC-149 modules; SPEC-149 handlers/tests are fmt-clean and unit/e2e covered |

## Confidence

High for single-replica auth deployments (demo GCP compose). Medium until multi-replica bus exists.

## Verification honesty (2026-09-19)

| Gate | Result |
|------|--------|
| `cargo test -p edgequake-api --lib handlers::websocket` | 18 passed |
| `cargo test -p edgequake-api --test e2e_spec149_realtime_progress` | 6 passed |
| Vitest (`websocket/__tests__` + `stream-client-spec149`) | 9 passed |
| Playwright `e2e/spec149-real-time-update.spec.ts` | 2 passed |
| `pnpm typecheck` | passed |
| `make test-e2e-lint` | passed |
| `cargo fmt --check` (SPEC-149 Rust files) | passed |
| `cargo clippy -p edgequake-api --all-targets -- -D warnings` | blocked by pre-existing non-SPEC-149 warnings |
