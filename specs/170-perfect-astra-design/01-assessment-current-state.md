# 01 — Current-State Assessment (grounded)

> Hub: [README](./README.md) · Prev: [00 WHY](./00-why-first-principles.md) · Next: [02 Product Owner](./02-lens-product-owner.md)

Every finding below is traced to a file or to a measured number from the
Playwright harness ([`e2e/metrics.mjs`](./e2e/metrics.mjs)). Baseline artifacts:
[`e2e/screenshots/baseline/metrics.json`](./e2e/screenshots/baseline/metrics.json).

## 1. What exists today (inventory)

Homepage composition — [`index.astro`](../../edgequake-website/src/pages/index.astro):

```
MarketingLayout
 ├─ Header            sticky, blur, logo + 5 nav links + theme + GitHub + CTA
 └─ main
    ├─ Hero           2-col: copy + badges + CTAs  |  product-hero.png
    ├─ Problem        "Similarity search loses the thread" · 6 red cards
    ├─ Solution       "Built for real Graph-RAG work" · 6 link cards
    ├─ Architecture   inline SVG pipeline (desktop) / vertical list (mobile)
    ├─ Benchmarks     4 stats + 3-row Acc table + <details> micro-bench
    ├─ QuickStart     3 tabs (Docker/REST/UI) · code blocks
    ├─ Ecosystem      4 surfaces + 8 crate cards
    └─ EnterpriseCTA  3 pillars + "Talk to the Team" + trust row
 └─ Footer            4 link groups + brand line
```

Design tokens live in [`global.css`](../../edgequake-website/src/styles/global.css)
as a well-structured CSS-variable system (`--mkt-*`) with light/dark overrides —
**this is good and we keep/extend it**.

## 2. Measured baseline (what the machine sees)

Captured on the running dev server at `localhost:4321`, 2× DPR, light+dark,
desktop (1440×900) and mobile (390×844):

| View | CLS | Horizontal overflow | Overflowers | Notes |
|------|-----|---------------------|-------------|-------|
| desktop / light | 0.0023 | 0 px | 1 | hero glow blob |
| desktop / **dark** | **0.0229** | 0 px | 1 | **font-swap reflow** |
| mobile / light | 0.0022 | 0 px | 12 | code blocks + glow |
| mobile / dark  | 0.0065 | 0 px | 12 | code blocks + glow |

### 2.1 Root-cause of the dark-mode CLS (0.023)
Shift sources reported by `PerformanceObserver`:
`DIV.mt-8, DIV.mt-16, DIV.mt-6, #text` — i.e. **hero/body text reflowing when the
web font swaps in**. Cause: fonts are loaded render-non-blocking via a Google
Fonts `<link ... display=swap>` in
[`MarketingLayout.astro`](../../edgequake-website/src/layouts/MarketingLayout.astro),
with **no metric-matched fallback**, so the fallback→web-font swap changes line
box heights and pushes content.

```
   load ──▶ paint (fallback font)  ──swap──▶ repaint (Source Serif/Sans)
             │  line-height = A                 │  line-height = B  (B≠A)
             └──────────────── CONTENT JUMPS ───┘   = CLS 0.023 (dark)
   Fix: @font-face fallbacks with size-adjust/ascent-override so A == B → CLS 0.
```

### 2.2 Overflowers
- `div.absolute.top-0` (hero glow) extends past the viewport right edge
  (`right:1528` at vw 1440; `right:478` at vw 390). Contained by body
  `overflow-x-hidden`, so no scrollbar — but it is sloppy and should be clipped at
  the section, not the body.
- On mobile, QuickStart `code / .ec-line` report `right:953` at vw 390 — code
  scrolls internally (`overflow-x-auto`), acceptable but the content model should
  not rely on the body clip.

## 3. Design critique (grounded, by symptom → cause → principle)

| # | Symptom (observable) | Cause (in code) | Violates |
|---|----------------------|-----------------|----------|
| C1 | Monotonous page: 8 bands, all centered heading + subhead + grid | Same template repeated in every `sections/*.astro` | [P3](./00-why-first-principles.md) |
| C2 | Zero motion/polish | [`FadeIn.astro`](../../edgequake-website/src/components/FadeIn.astro) ignores its `delay` prop and renders a plain `<div>` — **dead abstraction** | [P4](./00-why-first-principles.md), DRY |
| C3 | Text jumps on load (dark) | No metric-matched font fallback (§2.1) | [P2](./00-why-first-principles.md) |
| C4 | Product feels small/static | Single `object-cover` thumbnail, no framing/depth | [P5](./00-why-first-principles.md) |
| C5 | Heavy, flat cards | `.mkt-card` uses `0 24px 64px` shadow on light bg → muddy | [P4](./00-why-first-principles.md) |
| C6 | Repeated section-header markup | `text-center max-w-3xl mx-auto` + `font-display` duplicated in **all 8** sections | DRY |
| C7 | Repeated icon-path maps | `iconPaths` objects re-declared in Problem/Solution/EnterpriseCTA | DRY |
| C8 | Generic single-accent palette | Only one blue accent; no supporting hues for data/graph story | [P1](./00-why-first-principles.md) |
| C9 | Hero glow escapes bounds | `absolute … translate-x-1/4` relies on body clip (§2.2) | craft |

## 4. What is already good (keep / amplify — avoid regressions)

- **Honest tone** in Benchmarks (refuses to overclaim) — a trust asset ([P1](./00-why-first-principles.md)).
- **Token system** (`--mkt-*`, light/dark) — extend, don't replace.
- **Accessibility scaffolding**: skip-link, `:focus-visible`, `prefers-reduced-motion`
  block, ARIA tabs in QuickStart, semantic landmarks.
- **SEO/JSON-LD** in the layout — must be preserved verbatim.
- **Build-time data**: GitHub stars + VERSION ([`lib/`](../../edgequake-website/src/lib))
  with safe fallbacks — see [04](./04-lens-database-data-layer.md).

## 5. Baseline screenshots

Full-page captures (light+dark, desktop+mobile) are in
[`e2e/screenshots/baseline/`](./e2e/screenshots/baseline). These are the
before-state for the iteration log in
[11 Metrics & Acceptance](./11-metrics-and-acceptance.md).

> **Conclusion:** the site is *functional and honest* but *undifferentiated*. The
> redesign keeps the honesty, the tokens, and the a11y; fixes the measurable CLS;
> and replaces monotony + dead motion with shaped rhythm, real reveals, and
> product-forward framing. Requirements follow in [08](./08-specification.md).
