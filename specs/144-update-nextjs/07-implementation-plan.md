# 07 — Implementation Plan

## Phase A — Spec pack (this directory)

Done when README + laws + lenses + matrices land.

## Phase B1 — Dependency bump

1. `pnpm view next version` — confirm ≥16.3.3.
2. `pnpm add next@16.3.3` and `pnpm add -D eslint-config-next@16.3.3`.
3. Keep React 19.2.x unless peers force a patch.
4. `pnpm typecheck` + `pnpm test`.

## Phase B2 — Unify proxy

1. Refactor `src/proxy.ts` to compose `swaggerSlash` + `authGuard`.
2. Combined `config.matcher`.
3. Delete root `middleware.ts`.
4. Auth + swagger e2e green.

## Phase B3 — Build parity

1. Dockerfile: `npx next build --webpack`.
2. Preserve `compress: false`, proxy limits.
3. Document Turbopack NFT re-try as follow-up.

## Phase B4 — Free 16.3 wins

1. No Instant flags yet.
2. `pnpm run build` + critical e2e.

## Phase C — Instant Navigations (allowlisted)

1. Ensure `/` and `/documents` have loading shells.
2. Enable `cacheComponents` + `partialPrefetching`.
3. Add `@next/playwright` + `instant()` tests for allowlist.
4. Keep document detail / query / graph outside allowlist.

## Phase D — Gates

See [08-e2e-test-matrix.md](08-e2e-test-matrix.md).

## Order

```ascii
  A docs → B1 bump → B2 proxy → B3 Docker → B4 verify
                                          │
                                          └─ C instant (if B green) → D e2e
```

## DRY / SOLID

- Helpers for auth/swagger; one facade.
- Upload limit constants remain imported SSOT.
- No duplicated Next config across Docker and local without shared intent.

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
