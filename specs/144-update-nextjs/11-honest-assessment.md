# 11 — Honest Assessment

## What this cut actually buys

- Active LTS patch line (16.3.3) with free memory/SSR/prefetch improvements.
- Cleaner Next 16 network boundary (`proxy` SSOT).
- Build path honesty between Docker and local.
- A controlled on-ramp for Instant Navigations later.

## What it does not buy

- Automatic SPA-feel everywhere (flags stay off until shells exist).
- TypeScript 7 speedups.
- Turbopack production default (webpack remains until NFT proven).
- Any backend accuracy or ingestion improvements.

## Residual risks

| Risk | Residual? | Notes |
|------|-----------|-------|
| Turbopack NFT still broken on 16.3.3 | Yes | Keep webpack; re-test later |
| Instant Navigations build strictness | **Yes — blocked** | `cacheComponents` + react@19.2.3 → `unstable_postpone` missing on webpack prerender |
| Codemod surprise | Low | Manual pin preferred |
| Auth matcher drift | Medium | Covered by auth e2e + proxy-guards unit |

## Instant Navigations status (honest)

Phase C **prepared but flags off**:

- Shells + `instant` segment markers + `@next/playwright` are in tree.
- Enabling `cacheComponents` / `partialPrefetching` failed production build
  (`React.unstable_postpone is not defined` on `/_not-found`).
- Free 16.3 wins still apply without those flags.
- Re-enable when vendor pin supports postpone; then lock with `instant()` e2e.

## Confidence

High for B1–B4 (bump + proxy + parity + webpack build). Instant Navigations
flags deferred with documented blocker (not silently claimed done).

## Cross-refs

- Acceptance: [10-acceptance.md](10-acceptance.md)
- WHY: [00-why.md](00-why.md)
