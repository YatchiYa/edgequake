# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Security floor ≥16.2.11 | SPEC-085 PKG-next; July 2026 GHSAs |
| Target 16.3.3 | [nextjs.org/blog/next-16-3](https://nextjs.org/blog/next-16-3); npm `next@16.3.3`; also ships August 2026 Critical GHSAs ([GHSA-p293-qw3h-jr36](https://github.com/vercel/next.js/security/advisories/GHSA-p293-qw3h-jr36), [GHSA-2xp9-vwfh-vxw4](https://github.com/vercel/next.js/security/advisories/GHSA-2xp9-vwfh-vxw4)) |
| middleware → proxy | [Upgrade guide v16](https://nextjs.org/docs/app/guides/upgrading/version-16) |
| Auth cookie guard | SPEC-083 X-27; `edgequake_access_token` |
| Swagger trailing slash | `src/proxy.ts` + `skipTrailingSlashRedirect` |
| Upload proxy limits | SPEC-038; `proxyTimeout` / `proxyClientMaxBodySize` |
| SSE no gzip | `next.config.ts` `compress: false` |
| Webpack NFT pin | SPEC-085; `safe-build.sh --webpack` |
| Instant Navigations | [Instant Navigations guide](https://nextjs.org/docs/app/guides/instant-navigation) |
| PDF/MD sync regression | SPEC-143 |

## Code SSOT (as-is → target)

| Concern | As-is | Target |
|---------|-------|--------|
| Next version | `16.2.11` | `16.3.3` |
| eslint-config-next | `16.2.11` | `16.3.3` |
| Auth boundary | root `middleware.ts` | `src/proxy.ts` `authGuard` |
| Swagger slash | `src/proxy.ts` only | same file, composed |
| Instant Navigations | off | Phase C allowlist only |
| Docker build | `npx next build` | `npx next build --webpack` |
| Local build | `next build --webpack` | unchanged until NFT proven |

## Related specs

| Spec | Relationship |
|------|--------------|
| SPEC-085 | Floor + webpack decision; raise pin |
| SPEC-083 X-27 | Auth guard semantics preserved |
| SPEC-038 | Proxy body/timeout preserved |
| SPEC-017 | DRY/SOLID — one boundary module |
| SPEC-143 | Must remain green |
| SPEC-128 | PDF worker / overlay smoke |

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
