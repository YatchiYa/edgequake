# 09 — Implementation Plan (DRY / SOLID / First-Principles)

> Hub: [README](./README.md) · Prev: [08 Specification](./08-specification.md) · Next: [10 Edge Cases & Testing](./10-edge-cases-and-testing.md)

Implements [08](./08-specification.md). Each phase is independently verifiable and
leaves the site shippable (incremental, zero-regression).

## 0. Guiding constraints

- Static Astro, **no new runtime deps**. Motion = CSS + ~1 KB IO snippet.
- Extend `--mkt-*` tokens; never fork the system.
- Every new file has **one** responsibility (SRP); sections **compose** primitives.

## 1. Component plan (target tree)

```
src/components/
├─ primitives/
│  ├─ Section.astro        props: id?, tone?("base"|"alt"), focal?(bool)  → <section> + rhythm
│  ├─ SectionHeader.astro  props: eyebrow?, title, lede?, align?          → eyebrow+h2+lede
│  ├─ Reveal.astro         props: as?, delay?, class?                     → reveal wrapper
│  ├─ Icon.astro           props: name(keyof registry), class?            → <svg><path/></svg>
│  ├─ Button.astro         props: href, variant("primary"|"outline"), …  → <a> button
│  ├─ Badge.astro          props: (slot)                                  → pill
│  └─ Stat.astro           props: value, label                           → number+caption
├─ data/icons.ts           single icon-path registry (DIP target)
├─ sections/*.astro        refactored to compose the above
├─ Header.astro Footer.astro
└─ FadeIn.astro            DELETED
```

### SOLID checkpoints
- **SRP**: a primitive that renders *and* animates *and* fetches = split it.
- **OCP**: new tone/variant = new prop value + token, not an edit to logic.
- **DIP**: sections import `Icon`/`icons.ts`, never raw `d="…"` strings.

## 2. Phases

### Phase 1 — Foundation (tokens + fonts + motion) → fixes REQ-170-01/02/05/14
1. In [`global.css`](../../edgequake-website/src/styles/global.css): add
   metric-matched `@font-face` fallbacks (`Source Serif 4 Fallback` over Georgia,
   `Source Sans 3 Fallback` over system-ui) with `size-adjust`/`ascent-override`/
   `descent-override`/`line-gap-override`; update `--font-display`/`--font-sans`
   stacks to include them.
2. Add tokens: `--mkt-ring`, `--mkt-elevate`, `--mkt-grid`, `--mkt-accent-2`,
   band padding utilities; replace heavy `.mkt-card` shadow with two-layer elevation.
3. Add `.reveal` CSS (hidden/visible/reduced-motion) + `.mkt-grid-bg` texture util.
4. **Measure CLS** → expect dark-mode 0.023 → ~0. Gate before proceeding.

### Phase 2 — Primitives → fixes REQ-170-10/11/12/13
1. Create `data/icons.ts` (union of all icon paths used today).
2. Create `Icon`, `SectionHeader`, `Section`, `Reveal`, `Button`, `Badge`, `Stat`.
3. Add the IO reveal script (once, in layout or an inline island); default-visible.
4. Delete `FadeIn.astro` after migrating usages.

### Phase 3 — Hero (focal #1) → fixes REQ-170-20/21/25, C4/C9
1. Rebuild `Hero.astro` using primitives: eyebrow, fluid H1, lede, `Button`s.
2. Replace thumbnail with framed **real product graph** (`/public/product-graph.png`
   promoted from `e2e/screenshots/product/graph.png`), browser-frame + elevation,
   intrinsic `width/height`, `loading="eager"`.
3. Contain hero glow in-section (`overflow-clip`), `aria-hidden`; add faint grid.
4. Add quiet proof strip (Apache 2.0 · stars · `v{version}` · `{QUERY_MODES.length}` modes).

### Phase 4 — Remaining bands → fixes REQ-170-10/22/24, C1/C5/C6
Refactor Problem, Solution, Architecture, Benchmarks, QuickStart, Ecosystem,
EnterpriseCTA to compose `Section`+`SectionHeader`+`Icon`+`Reveal`; apply
alternating tone; differentiate focal density for Benchmarks & QuickStart;
elevate cards. Keep all copy/data and the honest benchmark framing.

### Phase 5 — Promote assets & wire data → fixes REQ-170-15/03
1. Copy chosen product PNGs into `public/` with recorded dimensions.
2. Derive "six modes" from `QUERY_MODES.length`; keep `getProductVersion()`.

### Phase 6 — Iterate to perfect → [11](./11-metrics-and-acceptance.md)
Re-capture `after/` screenshots + metrics; fix any CLS/overflow/contrast/tap
regressions; repeat until all MUST gates green.

### Phase 7 — Validate
`astro check` + `astro build`; spot-check a `/docs` page for CSS leakage; confirm
offline build (fallbacks).

## 3. DRY wins (explicit)

| Duplication removed | Before | After |
|---------------------|--------|-------|
| Section header markup | 8× `text-center max-w-3xl…` | 1× `SectionHeader` |
| Icon path maps | 3× `iconPaths` objects | 1× `icons.ts` + `Icon` |
| Buttons | ad-hoc per section | `Button` variants |
| "Six modes" literal | hardcoded "Six/six" | `QUERY_MODES.length` |
| Motion | dead `FadeIn` | real `Reveal` |

## 4. Rollback / safety

Work is additive + component-scoped; each phase is a coherent commit. If a phase
regresses metrics, revert that phase only. The token layer is backward-compatible
(old `.mkt-*` classes keep working), so partial application never breaks the page.

## 5. Task checklist (traceable)

- [ ] T1 fonts fallback + tokens + card elevation (REQ-01/02/14/22)
- [ ] T2 reveal CSS + IO script (REQ-05/12)
- [ ] T3 icons.ts + Icon (REQ-11)
- [ ] T4 Section/SectionHeader/Button/Badge/Stat (REQ-10/13)
- [ ] T5 Hero rebuild + framed product shot + proof strip (REQ-20/21/25)
- [ ] T6 refactor 7 remaining sections (REQ-10/22/24)
- [ ] T7 promote assets, derive modes/version (REQ-03/15)
- [ ] T8 delete FadeIn, grep DRY checks (REQ-10/11/12)
- [ ] T9 iterate metrics to CLS 0 (REQ-01..04)
- [ ] T10 astro check/build + a11y audit (REQ-30..34/43)

Edge cases that shape these tasks → [10](./10-edge-cases-and-testing.md).
