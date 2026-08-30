# 09 — Edge Cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| E1 | Auth disabled | Proxy no-ops auth; swagger still works | G5 / G8 |
| E2 | Auth enabled, no cookie | Redirect to login with `redirect` param | G5 |
| E3 | Exact `/swagger-ui` (no slash) | 307 to `/swagger-ui/` | G6 |
| E4 | SSE query under Next proxy | `compress: false` preserved | G4 |
| E5 | Large PDF upload | `proxyTimeout` + body size numeric | upload e2e / typecheck |
| E6 | Turbopack NFT ENOENT | Keep `--webpack` on Docker + safe-build | G2 |
| E7 | Instant flags + unguarded dynamic | Do not enable globally; allowlist only | G9 / build |
| E8 | Node &lt; 20.9 | Document engines; CI/Docker pin 20 | CI |
| E9 | Stale `bun.lock` | pnpm SSOT; ignore bun.lock | SPEC-085 |
| E10 | SPEC-143 sync after bump | Continuous stack + sync lock | G7 |
| E11 | Dual middleware leftover | Delete root `middleware.ts` | G5 + file absent |
| E12 | Prefetch inlining surprises | Smoke nav; no product change expected | G3 / G8 |

## Cross-refs

- E2E matrix: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
- NextJS lens: [05-lenses/006-nextjs-expert.md](05-lenses/006-nextjs-expert.md)
