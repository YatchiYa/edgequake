# 04 — Lens: Database / Data-Layer Expert

> Hub: [README](./README.md) · Prev: [03 Full-Stack](./03-lens-fullstack.md) · Next: [05 UX/UI](./05-lens-ux-ui.md)

A static marketing site has no runtime database — but it **does** have a
build-time *data layer*: the inputs that render the page must be **sourced,
cached, versioned, and have safe fallbacks**, exactly like a query layer. Treating
it this way is what keeps the site accurate and never-broken.

## 1. The data layer, mapped

```
   SOURCE (authoritative)          ADAPTER (build time)         RENDERED FACT
   ─────────────────────           ────────────────────         ─────────────
   repo-root VERSION ────────────▶ lib/product.ts ───────────▶ badges, crate vX, schema.org
     fallback "0.36.0"             getProductVersion()          QUERY_MODES[6]
   GitHub REST /repos ───────────▶ lib/github.ts ────────────▶ "N GitHub stars"
     fallback 2275 (floor)         getGithubStats()             (Math.max → never regress)
   Markdown content/* ───────────▶ content.config.ts ────────▶ tutorial pages
   demo.edgequake.com (product) ─▶ e2e/capture.mjs (Playwright)▶ /public screenshots
```

### Observations (grounded)
- [`lib/product.ts`](../../edgequake-website/src/lib/product.ts) reads `VERSION`
  with a **regex-validated** parse and a pinned fallback — deterministic.
- [`lib/github.ts`](../../edgequake-website/src/lib/github.ts) fetches **once per
  build**, clamps `Math.max(stars, 2275)` so a rate-limited build **never shows a
  lower number** — a correct "monotonic read" design.
- `QUERY_MODES` is the single source for the "six modes" claim — the Hero caption
  and Ecosystem must both derive from it (DRY), not hardcode "six".

## 2. Data-integrity requirements (treat like constraints)

| ID | Constraint | Enforcement |
|----|-----------|-------------|
| D-1 | Version shown == repo `VERSION` (or pinned fallback) | existing `getProductVersion()`; **reuse, don't re-read** |
| D-2 | Star count never regresses within a build | existing `Math.max` floor |
| D-3 | "Six modes" derives from `QUERY_MODES.length` | refactor literals → derive |
| D-4 | Every screenshot file referenced exists in `/public` | build + link check ([10](./10-edge-cases-and-testing.md)) |
| D-5 | No secret in client output (GITHUB_TOKEN build-only) | already server-only; keep |
| D-6 | Benchmarks numbers match `docs/comparisons` methodology | copy sourced, link retained |

## 3. Screenshots: provenance, caching, refresh

**Status:** the live demo's interior pages are behind a **login wall** (verified:
`/documents`, `/query`, `/graph` redirect to `/login` for anonymous visitors). A
member account was provisioned via the demo **invite/magic-link** and the five
core screens were captured **authenticated** at 3200×2000 into
[`e2e/screenshots/product/`](./e2e/screenshots/product). The session uses
**single-use refresh-token rotation**, so all captures must run inside **one**
fresh-login context (reusing a saved `storageState` logs out). Harness:
[`e2e/capture-product.mjs`](./e2e/capture-product.mjs).

**Policy (deterministic + honest):**
1. **Authenticated interiors** → fresh Playwright captures
   (`DEMO_USER`/`DEMO_PASS`, one session) → `e2e/screenshots/product/*.png`.
2. **Promotion** → chosen frames are copied into
   [`public/`](../../edgequake-website/public) with intrinsic dimensions recorded.
3. **Repo fallback** → the authoritative demo-sourced captures in
   [`public/tutorial/`](../../edgequake-website/public/tutorial) remain valid when
   the demo is unreachable.

```
  anon ──▶ demo.edgequake.com/documents ──▶ 302 /login  (wall)
  auth ──▶ (creds) ──▶ interior render ──▶ Playwright PNG ──▶ /public/*.png
  today ─▶ reuse public/tutorial/*.png  (authentic, demo-sourced, versioned in repo)
```

**Image hygiene (prevents CLS — ties to [P2](./00-why-first-principles.md)):**
- Every `<img>` MUST carry intrinsic `width`+`height` (or `aspect-ratio`) so the
  box is reserved before decode → **no image-induced CLS**.
- Hero image `loading="eager"`, below-fold `loading="lazy"` + `decoding="async"`.
- Serve appropriately sized assets; the current `product-hero.png` is 3084×1530
  for a 1200-wide slot — consider a right-sized/`webp` variant (performance, not
  correctness).

## 4. Determinism & offline builds

The site must build with **no network** (CI, air-gapped): `github.ts` already
falls back; `product.ts` already falls back. The redesign must not add any
build-time fetch without a fallback. This is the data-layer equivalent of
"degrade gracefully."

## 5. Cross-refs
- Build-time flow detail → [03 Full-Stack §4](./03-lens-fullstack.md).
- Image-CLS gate → [11 Metrics §Image stability](./11-metrics-and-acceptance.md).
- Broken-link / missing-asset edge cases → [10](./10-edge-cases-and-testing.md).
