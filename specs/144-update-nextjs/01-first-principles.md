# 01 — First Principles (LAW-144)

## Axioms

| ID | Law | Operational meaning |
|----|-----|---------------------|
| **LAW-144-1** | Security floor never regresses | Pin ≥ patched Active LTS; never drop below 16.2.11; raise pin to 16.3.3 |
| **LAW-144-2** | One network boundary | Single `src/proxy.ts` composes auth + swagger; no dual middleware/proxy SSOT |
| **LAW-144-3** | Opt-in features need strategy | Instant Navigations / `'use cache'` only where Suspense or cache strategy exists |
| **LAW-144-4** | SSE must stream | `compress: false` until reverse-proxy excludes `text/event-stream` |
| **LAW-144-5** | Build path parity | Docker `next build` ≡ `safe-build.sh` bundler choice (`--webpack` until NFT green) |
| **LAW-144-6** | Unfakable contracts | E2E asserts UI/network observables — not package.json version alone |
| **LAW-144-7** | Instant Navigations allowlisted | Route allowlist + `@next/playwright` `instant()` locks shells |

## Anti-patterns

| Anti-pattern | Violates |
|--------------|----------|
| Stay on 16.2.11 “because it works” while Active LTS moves | LAW-144-1 |
| Keep both `middleware.ts` and `src/proxy.ts` as twin entrypoints | LAW-144-2 |
| Enable `cacheComponents` globally without shell audit | LAW-144-3 |
| Re-enable Next gzip and break SSE token streaming | LAW-144-4 |
| Local `--webpack` but Docker plain Turbopack build | LAW-144-5 |
| E2E that only checks `next --version` | LAW-144-6 |
| Claim Instant Navigations without `instant()` tests | LAW-144-7 |

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- NextJS lens: [05-lenses/006-nextjs-expert.md](05-lenses/006-nextjs-expert.md)
