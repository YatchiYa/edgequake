# Lens — Next.js Expert

## Version matrix

| Item | Value |
|------|-------|
| From | `16.2.11` |
| To | `16.3.3` (Active LTS; re-verify `pnpm view next version`) |
| React | `19.2.3` (stay on 19.2.x) |
| Node | `>=20.9.0` |
| Upgrade class | Minor within 16.x — not 15→16 major |

## Official docs (must follow)

| Topic | URL |
|-------|-----|
| 16.3 release | https://nextjs.org/blog/next-16-3 |
| Instant Navigations | https://nextjs.org/docs/app/guides/instant-navigation |
| Upgrade to 16 | https://nextjs.org/docs/app/guides/upgrading/version-16 |
| App-like 16.3 | https://nextjs.org/blog/building-app-like-experiences-with-nextjs-16-3 |
| Codemod | `npx @next/codemod@canary upgrade latest` (optional; prefer manual pin) |

## Free wins (no flags)

From 16.3 announcement — apply by bump alone:

- Turbopack memory eviction in `next dev`
- Disk cache for builds (when Turbopack used)
- Native Node.js streams in App Router SSR
- Prefetch inlining
- Versioned agent docs via `next dev`

## Opt-in (Phase C)

```ts
const nextConfig = {
  cacheComponents: true,
  partialPrefetching: true,
};
```

Requires Suspense / `'use cache'` / explicit blocking strategy per dynamic
access. Use `@next/playwright` `instant()` to lock shells.

## Experimental — off this cut

- `reactCompiler` + `experimental.turbopackRustReactCompiler`
- `experimental.useOffline`
- TypeScript 7 for `next build` typecheck

## middleware → proxy

Next 16 renames the convention. EdgeQuake target:

```ascii
  BEFORE: middleware.ts (auth) + src/proxy.ts (swagger)
  AFTER:  src/proxy.ts only (auth ∘ swagger)
```

Preserve matcher coverage for protected HTML routes and `/swagger-ui`.

## Async Request APIs

Already mostly migrated (client hooks + Promise params). Re-scan for sync
`cookies()` / `headers()` / `params` in Server Components after bump.

## Bundler note

Turbopack is default for `next build` on 16. Keep `--webpack` until
`output: "standalone"` NFT for proxy/middleware artifacts is proven on 16.3.3.

## Cross-refs

- Laws: [01-first-principles.md](../01-first-principles.md)
- SPEC-085 PKG-next security floor
