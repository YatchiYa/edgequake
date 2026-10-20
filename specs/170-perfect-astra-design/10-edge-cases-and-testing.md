# 10 — Edge Cases & Testing

> Hub: [README](./README.md) · Prev: [09 Implementation Plan](./09-implementation-plan.md) · Next: [11 Metrics & Acceptance](./11-metrics-and-acceptance.md)

First-principles QA: enumerate every way the page can break, state the mitigation
*in the design*, and name the test. "Envisioned → mitigated → tested."

## 1. Layout / CLS edge cases

| # | Edge case | Mitigation | Test |
|---|-----------|------------|------|
| E1 | Web font loads late → text reflow | metric-matched fallbacks (same line-box) | CLS metric dark+light ([11 G1](./11-metrics-and-acceptance.md)) |
| E2 | Hero image slow/decoding → jump | intrinsic `width`/`height`, `aspect-ratio` reserved box | image-CLS check |
| E3 | GitHub stars string length varies ("2,275" vs "10,000") | fixed badge min-width / `tabular-nums`; no layout dependence | width sweep |
| E4 | Reveal runs before box reserved | reveal animates transform/opacity only; box always reserved | CLS metric |
| E5 | Very long word/URL in code block | `overflow-wrap:anywhere` + in-surface x-scroll | mobile overflow metric |
| E6 | Glow/grid bleeds past viewport | `overflow-clip` on the section, not body | overflow=0 at 320–1920 |
| E7 | 320px ultra-narrow | fluid clamp type + 1-col grids; no fixed min-width wider than 320 | capture @320 |
| E8 | Ultra-wide 2560px | `max-w-*` caps content; centered | capture @2560 |

## 2. Theme / color edge cases

| # | Edge case | Mitigation | Test |
|---|-----------|------------|------|
| E9 | FOUC: wrong theme before hydration | existing inline pre-paint theme script (keep) | load flash check |
| E10 | Low contrast `mkt-text-dim` on `mkt-bg-alt` | audit + bump token if < 4.5:1 | contrast gate ([11 G3](./11-metrics-and-acceptance.md)) |
| E11 | `theme-color` meta out of sync | existing toggle updates meta (keep) | manual toggle |
| E12 | Secondary accent reduces contrast | restrict `--mkt-accent-2` to large/graphic use; verify | contrast gate |

## 3. Motion / JS edge cases

| # | Edge case | Mitigation | Test |
|---|-----------|------------|------|
| E13 | JS disabled → reveal hides content forever | default-visible; JS only adds hidden→animate | JS-disabled capture (visible) |
| E14 | `prefers-reduced-motion` | reveal + micro-interactions disabled | emulate reduced-motion capture |
| E15 | IO unsupported (very old browser) | feature-detect; else show all | guarded script |
| E16 | Reveal double-init / hot reload | idempotent `data-reveal-bound` guard | re-run harness |
| E17 | QuickStart tabs keyboard | preserve existing ←/→/Home/End + `aria-selected` | keyboard test |

## 4. Data / content edge cases

| # | Edge case | Mitigation | Test |
|---|-----------|------------|------|
| E18 | GitHub API rate-limited/offline | `github.ts` fallback 2275 floor (exists) | offline build |
| E19 | `VERSION` unreadable/malformed | `product.ts` regex + pinned fallback (exists) | temp-rename VERSION build |
| E20 | Referenced screenshot missing in `/public` | build/link check; keep repo fallback imagery | asset existence check |
| E21 | Stars drop below floor on rebuild | `Math.max(..,2275)` monotonic (exists) | n/a (by construction) |
| E22 | "Six modes" diverges from engine | derive from `QUERY_MODES.length` | grep literal "six" |

## 5. Accessibility edge cases

| # | Edge case | Mitigation | Test |
|---|-----------|------------|------|
| E23 | Mobile tap target < 44px | enforce min size on nav/toggle/CTA | tap-size metric |
| E24 | Focus lost in mobile menu | preserve focus handling; visible focus ring | keyboard test |
| E25 | Heading level skip after refactor | SectionHeader emits `h2`; sections never inline-bump | axe/review |
| E26 | Decorative SVG announced by SR | `aria-hidden="true"` on glows/grid/icons-in-labeled-ctx | review |

## 6. Build / integration edge cases

| # | Edge case | Mitigation | Test |
|---|-----------|------------|------|
| E27 | Marketing CSS leaks into Starlight `/docs` | `@layer` + `.mkt-*` scoping; spot-check docs page | docs capture diff |
| E28 | Tailwind 4 purge drops reveal/grid classes | classes used in markup or safelisted via tokens | build + visual |
| E29 | New `@font-face` fails to load | fallback stack renders identical metrics → invisible failure | CLS stays 0 offline |
| E30 | Deleting FadeIn breaks an import | grep usages before delete; migrate all | `astro check` |

## 7. Test harnesses (how each gate runs)

- **Layout/CLS/overflow/geometry** → [`e2e/metrics.mjs`](./e2e/metrics.mjs)
  (PerformanceObserver CLS, overflow scan, section geometry) across
  desktop/mobile × light/dark.
- **Reduced-motion & JS-disabled** → Playwright context flags
  (`reducedMotion:"reduce"`, `javaScriptEnabled:false`) capture — content visible.
- **Contrast & tap size** → DOM-computed color pairs + `getBoundingClientRect`
  in the harness (extendable), or `axe-core` injected.
- **Build** → `astro check` + `astro build` (+ one offline build with VERSION/token
  unset to exercise fallbacks).

All results recorded in [11 — Metrics & Acceptance](./11-metrics-and-acceptance.md).
