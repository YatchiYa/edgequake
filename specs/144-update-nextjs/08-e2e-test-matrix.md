# 08 — E2E Test Matrix

## Unfakable principle (LAW-144-6)

Assert observable UI, URLs, network streaming, and redirects — not
`package.json` version strings alone. Version pin is checked in unit/CI
assertions as a complement.

## Gate table

| ID | Spec / command | Asserts |
|----|----------------|---------|
| G0 | `pnpm typecheck` | Types compile on 16.3.3 |
| G1 | `pnpm test` | Unit suite |
| G2 | `pnpm run build` | Webpack standalone build |
| G3 | `ooda-228-critical-path.spec.ts` | Core product path |
| G4 | `streaming-test.spec.ts` | SSE tokens stream (`compress: false`) |
| G5 | `issue-180-auth-runtime-hardening.spec.ts` | Auth cookie / redirect via proxy |
| G6 | `api-explorer.spec.ts` | Swagger trailing slash / assets |
| G7 | `spec143-pdf-markdown-sync.spec.ts` | PDF/MD sync no regression |
| G8 | `spec144-next-upgrade-smoke.spec.ts` | Boot, nav shell, health/proxy; pin check |
| G9 | Phase C `instant()` cases | Allowlisted shells appear without waiting network |

## New smoke spec outline

```ts
// e2e/spec144-next-upgrade-smoke.spec.ts
// 1. GET / → dashboard chrome visible
// 2. Navigate /documents → list or empty state
// 3. /swagger-ui redirects to /swagger-ui/ (or loads assets)
// 4. Optional: read next version from a test-only endpoint OR
//    assert process env baked in NEXT_PUBLIC_APP_VERSION still boots
```

Version pin also asserted via a small vitest reading `package.json`.

## Auth matrix

| Mode            | Expect                                         |
| -----------------| ------------------------------------------------|
| Auth off (demo) | Public routes open                             |
| Auth on         | Unauthenticated deep link → `/login?redirect=` |

## Cross-refs

- Edge cases: [09-edge-cases.md](09-edge-cases.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
