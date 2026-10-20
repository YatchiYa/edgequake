# 03 — Lens: Full-Stack Developer

> Hub: [README](./README.md) · Prev: [02 Product Owner](./02-lens-product-owner.md) · Next: [04 Data Layer](./04-lens-database-data-layer.md)

## 1. Stack facts (grounded)

- **Astro 7** static output (`output: "static"`) —
  [`astro.config.mjs`](../../edgequake-website/astro.config.mjs).
- **Tailwind 4** via `@tailwindcss/vite`, tokens in `@theme` + CSS vars —
  [`global.css`](../../edgequake-website/src/styles/global.css).
- **Starlight** owns `/docs`; marketing pages use
  [`MarketingLayout.astro`](../../edgequake-website/src/layouts/MarketingLayout.astro).
- **React 19** island available (`@astrojs/react`) but homepage is **zero-JS**
  except a tiny inline theme script and the QuickStart tab script.

Implication: the redesign stays **HTML/CSS-first**. Any motion must be CSS or a
~1 KB IntersectionObserver snippet — never a framework.

## 2. Target component architecture (SOLID)

```
src/components/
├─ primitives/                 ← NEW, single-responsibility, reused everywhere
│  ├─ Section.astro            ← <section> wrapper: id, tone(alt|base), padding rhythm
│  ├─ SectionHeader.astro      ← eyebrow + h2 + lede   (kills C6 duplication)
│  ├─ Reveal.astro             ← real, reduced-motion-safe scroll reveal (kills C2)
│  ├─ Icon.astro               ← one icon registry     (kills C7 duplication)
│  ├─ Badge.astro / Stat.astro ← atomic, tokenized
│  └─ Button.astro             ← primary / outline variants (one source)
├─ sections/                   ← now COMPOSE primitives + own only their content/data
│  ├─ Hero.astro  Problem.astro  Solution.astro  Architecture.astro
│  ├─ Benchmarks.astro  QuickStart.astro  Ecosystem.astro  EnterpriseCTA.astro
├─ Header.astro  Footer.astro  (refined to use primitives where sensible)
└─ FadeIn.astro  → DELETED (replaced by Reveal)
```

### SOLID mapping

| Principle | Application |
|-----------|-------------|
| **S**RP | `SectionHeader` only renders a header; `Reveal` only animates entrance; sections only own their copy/data. |
| **O**CP | `Section tone="alt"` and `Button variant="outline"` extend behavior via props, not by editing the primitive. |
| **L**SP | Any section can be wrapped by `Reveal` or placed in `Section` without special-casing. |
| **I**SP | Small, focused prop interfaces (`<Icon name>`, `<Badge>`, `<Stat value label>`) — no god-components. |
| **D**IP | Sections depend on the `Icon` *registry abstraction*, not on inline SVG path strings. |

## 3. The `Reveal` primitive (design)

Replaces the no-op `FadeIn`. Requirements:
- Reduced-motion: if `prefers-reduced-motion: reduce`, render fully visible, no
  transform (honors existing global rule).
- **CLS-safe**: the element occupies its final box *before* animating — animate
  only `opacity` and `transform` (compositor-only; never layout properties).
- No-JS fallback: if the IO script never runs, content is visible (progressive
  enhancement) — default state is visible unless JS marks it `data-reveal`.

```
  ┌─ Reveal (wrapper, final size reserved) ────────────┐
  │  initial:  opacity:0; translateY(12px)   (GPU only)│
  │  in-view:  opacity:1; translateY(0)  (IO toggles)  │
  │  reduced:  opacity:1; no transform                 │
  └────────────────────────────────────────────────────┘
       animates transform+opacity → **0 layout cost** → CLS unaffected
```

## 4. Data flow (build-time, deterministic)

```
VERSION ─────────▶ lib/product.ts ─┐
GitHub API ──────▶ lib/github.ts  ─┼─▶ Hero/Ecosystem/Benchmarks props ─▶ static HTML
(build only)       (fallbacks)     │   (no client fetch, no runtime)
content/* ───────▶ tutorial pages ─┘
```

No client-side data fetching is added. See [04](./04-lens-database-data-layer.md).

## 5. Font-swap CLS fix (engineering)

Root cause in [01 §2.1](./01-assessment-current-state.md). Fix options compared:

| Option | CLS | Risk | Choice |
|--------|-----|------|--------|
| `display=swap` + no fallback (today) | 0.023 | reflow | ✗ |
| `display=optional` | ~0 | may not show web font on slow nets | partial |
| **Metric-matched `@font-face` fallback (`size-adjust`, `ascent/descent-override`) + `preconnect`** | **0** | none; progressive | ✓ |

We define fallback faces (`"Source Serif 4 Fallback"` over Georgia; `"Source Sans 3
Fallback"` over system-ui) with adjusted metrics so the fallback box == web-font
box → swap causes **no reflow**. Keep Google Fonts link but add `preconnect`
(already present) + set the font stacks to `Web, Fallback, generic`.

## 6. Risks & mitigations (engineering)

| Risk | Mitigation | Test |
|------|------------|------|
| Tailwind 4 `@theme` + new vars collide | Namespacing `--mkt-*`; additive only | `astro check` + visual metrics |
| Starlight global CSS leakage into docs | Keep marketing rules under `@layer` + `.mkt-*` scoping | docs page spot-check |
| IO reveal breaks SSR/no-JS | Default-visible; enhance on load | JS-disabled capture |
| Metric overrides wrong → still shifts | Measure CLS before/after; tune overrides | [`metrics.mjs`](./e2e/metrics.mjs) |

Continue to [04 — Data Layer](./04-lens-database-data-layer.md).
