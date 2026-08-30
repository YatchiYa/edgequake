# Lens — Product Owner

## Outcome

Developers and users get a faster, safer WebUI runtime on current Active LTS
Next.js without losing document sync, streaming answers, auth redirects, or
API explorer. Instant navigations feel SPA-like on core list routes only when
proven.

## Jobs to be done

1. Stay on patched Active LTS (security + vendor support).
2. Ship free 16.3 wins (dev memory, SSR throughput) with zero product change.
3. Keep SPEC-143 PDF/Markdown sync working.
4. Keep query SSE streaming token-by-token.
5. Adopt Instant Navigations only where it improves list↔list nav.

## Non-goals (this cut)

- TypeScript 7 migration.
- Experimental React Compiler / offline mode.
- Global Instant Navigations on PDF detail / graph / query.
- Backend or DB changes.

## Success metrics

| Metric | Gate |
|--------|------|
| `next` pin = 16.3.3 | package.json + lockfile |
| Critical path e2e | Pass |
| SPEC-143 sync e2e | Pass |
| SSE streaming e2e | Pass |
| Auth + swagger e2e | Pass |
| No schema PR | Ship without DB migration |

## Risks

| Risk | Mitigation |
|------|------------|
| Instant Navigations breaks dynamic routes | Off by default; allowlist |
| Turbopack NFT regresses Docker | Keep `--webpack` parity |
| Dual proxy confusion | One `src/proxy.ts` |

## Cross-refs

- Acceptance: [10-acceptance.md](../10-acceptance.md)
- UX: [004-ux.md](004-ux.md)
