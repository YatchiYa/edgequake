# 00 — Five WHYs

## Problem statement

EdgeQuake WebUI is pinned to Next.js **16.2.11** (SPEC-085 security floor).
Active LTS has moved to **16.3.x** with lower dev memory, faster SSR streams,
and opt-in Instant Navigations. Staying on 16.2.11 forgoes free wins, leaves
a split `middleware.ts` / `src/proxy.ts` boundary, and risks Docker builds
diverging from the webpack-safe local path.

---

### WHY 1 — Why upgrade at all?

**Answer:** 16.3 ships default-on improvements (Turbopack memory eviction,
native Node SSR streams, prefetch inlining) and is the current Active LTS
patch line. Security policy is “latest patched Active LTS,” not “minimum
floor forever.”

---

### WHY 2 — Why is the network boundary split?

**Answer:** Next 16 deprecated `middleware` in favor of `proxy`. Auth stayed
in root `middleware.ts` (SPEC-083 X-27); swagger trailing-slash landed in
`src/proxy.ts`. Two entrypoints violate DRY and confuse Turbopack NFT tracing.

---

### WHY 3 — Why keep webpack for production build?

**Answer:** On 16.2.11, Turbopack + `output: "standalone"` failed with
`middleware.js.nft.json` ENOENT (SPEC-085). Until 16.3.3 proves NFT green,
`--webpack` remains the honest production path; Docker must match.

---

### WHY 4 — Why not flip Instant Navigations globally?

**Answer:** `cacheComponents` + `partialPrefetching` turn unguarded dynamic
access into build/runtime decisions. Heavy client routes (document PDF sync,
query SSE, graph) lack Suspense/`use cache` shells. Global enable = breakage.

---

### WHY 5 — Why is “version bump only” insufficient?

**Answer:** Regression risk lives in proxy composition, SSE compression,
swagger slash, auth redirects, and SPEC-143 sync — not in the semver string.
Unfakable e2e must assert observable behavior.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- As-is: [03-code-as-is.md](03-code-as-is.md)
