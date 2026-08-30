# SPEC-144 — Update Next.js (16.2.11 → 16.3.3)

> **Mission:** Move EdgeQuake WebUI to the latest Active LTS Next.js patch
> (`16.3.3`), take free runtime/dev wins, unify the network boundary under
> one `proxy` module (DRY/SOLID), and adopt Instant Navigations only on
> allowlisted routes — with unfakable e2e so SPEC-143 and critical paths
> do not regress.
>
> **Method:** Pin `next` + `eslint-config-next`; compose auth + swagger into
> `src/proxy.ts`; keep `compress: false` + webpack standalone until NFT proven;
> Instant Navigations opt-in after shell audit.
>
> **Target cut:** next patch after SPEC-143.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  LAW: Stay on patched Active LTS; one proxy SSOT; Instant Navigations        │
│       only where Suspense/`use cache` strategy exists and e2e locks it.      │
│                                                                              │
│  Product path:                                                               │
│    next@16.2.11 → next@16.3.3   (free: memory, SSR streams, prefetch)      │
│    middleware.ts + src/proxy.ts → single src/proxy.ts                        │
│    Instant Navigations          → allowlisted shells + instant() e2e         │
│                                                                              │
│  No DB migration. SSE compress:false stays. Webpack pin until NFT green.     │
│  SPEC-143 PDF/MD sync must remain green.                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Document map

```ascii
  README
    → 00-why (5 WHY)
    → 01-first-principles (LAW-144-1..7)
    → 02-cross-ref-matrix
    → 03-code-as-is
    → 04-target-architecture
    → 05-lenses/ (PO, fullstack, DB, UX, front, NextJS expert)
    → 06-ux-ui-spec
    → 07-implementation-plan
    → 08-e2e-test-matrix
    → 09-edge-cases
    → 10-acceptance
    → 11-honest-assessment
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| I1 | Bump `next` / `eslint-config-next` → 16.3.3 | Done |
| I2 | Unify `src/proxy.ts` (auth + swagger) | Done |
| I3 | Docker ≡ safe-build `--webpack` | Done |
| I4 | Free 16.3 wins verified (no Instant flags) | Done |
| I5 | Instant Navigations allowlist + shells | Prepared; flags deferred (postpone blocker) |
| T1 | Unfakable smoke + regression e2e | Done (port **3010**) |
| A1 | Acceptance | Done |

## Locked decisions

| Decision | Choice |
|----------|--------|
| Target | `next@16.3.3` + `eslint-config-next@16.3.3` |
| Instant Navigations | Off for bump; Phase C allowlisted only |
| TypeScript 7 | Out of scope |
| React Compiler / `useOffline` | Experimental — off |
| Network boundary | Single `src/proxy.ts`; delete root `middleware.ts` |
| Bundler | Keep `next build --webpack` until standalone NFT green |
| DB | No schema / migration |

## Official grounding

| Link | Role |
|------|------|
| [Next.js 16.3 blog](https://nextjs.org/blog/next-16-3) | Release notes / free wins |
| [Instant Navigations](https://nextjs.org/docs/app/guides/instant-navigation) | Opt-in SPA-feel nav |
| [Upgrade to v16](https://nextjs.org/docs/app/guides/upgrading/version-16) | middleware→proxy, async APIs |
| [App-like experiences](https://nextjs.org/blog/building-app-like-experiences-with-nextjs-16-3) | Cache Components adoption |
| [SPEC-085 PKG-next](../085-fix-security/packages/PKG-next.md) | Security floor ≥16.2.11 |

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-085](../085-fix-security/) | Security floor; webpack NFT decision |
| [SPEC-083 X-27](../083-improvements/) | Auth cookie / middleware guard |
| [SPEC-038](../038-*) | Large upload proxy timeout / body size |
| [SPEC-017](../017-dry-and-solid-audit/) | WebUI DRY/SOLID baseline |
| [SPEC-143](../143-view-pdf-markdown-sync-view/) | Must stay green post-bump |
| [SPEC-128](../128-layout-overlay/) | PDF worker / overlay regression surface |
