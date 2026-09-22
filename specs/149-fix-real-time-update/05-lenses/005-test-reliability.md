# Lens 005 — Test Reliability

## Mandatory proofs

| Layer | Must assert |
|-------|-------------|
| Rust e2e | Exact 101; receive `StageTransition` after subscribe; silence for foreign |
| Vitest | Production `ProgressWebSocket` + manager reset on token |
| Playwright | Frame received updates DOM with list polling aborted |

## Flake controls

- Bound waits with `expect.poll` / `waitForEvent`
- Do not depend on live LLM for unit/contract gates
- Mocked Playwright path for chromium gate; live `@load` optional

## Anti-patterns banned

- Asserting only that *some* status string changed without WS frame
- Reimplementing URL helpers inside the test file
- Skipping connect under Playwright for "stability" without an alternate explicit connect
