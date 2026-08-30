# 04 — Target Architecture

## Dependency target

```ascii
  package.json
    next                 16.3.3
    eslint-config-next   16.3.3
    react / react-dom    19.2.x (unchanged unless peer forces patch)
```

## Unified proxy (DRY facade)

```ascii
  request
    │
    ▼
  src/proxy.ts
    │
    ├─ swaggerSlash(req)  → 307 /swagger-ui/ if exact /swagger-ui
    │
    └─ authGuard(req)     → login redirect if auth required & no cookie
         │
         └─ NextResponse.next()
```

SOLID:

- **S:** `authGuard` and `swaggerSlash` are pure helpers; `proxy` only composes.
- **O:** New matchers add helpers without rewriting auth.
- **D:** Tests depend on helpers; Next only sees one `proxy` export.

Delete root `middleware.ts` after parity e2e.

## Config invariants

| Setting | Value | Why |
|---------|-------|-----|
| `compress` | `false` | LAW-144-4 SSE |
| `proxyTimeout` | 600_000 | Large PDF admit |
| `proxyClientMaxBodySize` | numeric bytes | SPEC-038 / #296 |
| `output` | `standalone` | Docker runtime |
| `cacheComponents` | Phase C only | LAW-144-3 |
| `partialPrefetching` | Phase C only | LAW-144-3 |

## Build parity

```ascii
  safe-build.sh  ──►  next build --webpack
  Dockerfile     ──►  next build --webpack   (aligned)
```

Re-evaluate Turbopack standalone NFT on 16.3.3 behind a documented gate;
do not flip default in this cut unless NFT proven.

## Instant Navigations (Phase C) — deferred flags

```ascii
  Prepared (shipped):
    loading.tsx shells for / and /documents
    @next/playwright installed (for future instant() e2e)

  Not enabled yet (build blocker):
    cacheComponents / partialPrefetching
    `export const instant` (requires cacheComponents — Next errors otherwise)
    Reason: next@16.3.3 + react@19.2.3 webpack prerender:
            "React.unstable_postpone is not defined"
```

Re-enable when Next/React pin ships postpone for Cache Components; then add
`instant()` e2e for allowlisted `/` ↔ `/documents`.

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edge cases: [09-edge-cases.md](09-edge-cases.md)
