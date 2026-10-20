# 08 — Specification (normative)

> Hub: [README](./README.md) · Prev: [07 Marketing/Growth](./07-lens-marketing-growth.md) · Next: [09 Implementation Plan](./09-implementation-plan.md)

Requirements use **MUST / SHOULD / MAY** (RFC-2119). Each has an ID `REQ-170-xx`,
a source lens, and a verification in [11](./11-metrics-and-acceptance.md).

## A. Layout stability & performance

| ID | Requirement | Verify |
|----|-------------|--------|
| REQ-170-01 | The homepage **MUST** have **CLS = 0** on {desktop,mobile}×{light,dark}. | [11 G1](./11-metrics-and-acceptance.md) |
| REQ-170-02 | Web-font load **MUST NOT** cause reflow — metric-matched `@font-face` fallbacks required. | [11 G1](./11-metrics-and-acceptance.md) |
| REQ-170-03 | Every `<img>` **MUST** declare intrinsic `width`+`height` (or `aspect-ratio`). | [11 G2](./11-metrics-and-acceptance.md) |
| REQ-170-04 | No element **MUST** cause unintended horizontal overflow; decorative glows **MUST** be clipped at their section (`overflow-clip`), not the body. | [11 G2](./11-metrics-and-acceptance.md) |
| REQ-170-05 | Motion **MUST** animate only `opacity`/`transform`; **MUST** be disabled under `prefers-reduced-motion`. | [11 G4](./11-metrics-and-acceptance.md) |

## B. Structure, DRY & SOLID

| ID | Requirement | Verify |
|----|-------------|--------|
| REQ-170-10 | A single `SectionHeader` primitive **MUST** render all band headers (eyebrow+h2+lede); no section re-implements it. | grep + review |
| REQ-170-11 | A single `Icon` registry **MUST** supply all inline SVG icons; `iconPaths` duplication removed. | grep |
| REQ-170-12 | A real `Reveal` primitive **MUST** replace `FadeIn`; `FadeIn.astro` deleted. | grep + no-JS test |
| REQ-170-13 | Section content (problems, features, crates, etc.) **MUST** remain data-driven arrays. | review |
| REQ-170-14 | Design values **MUST** come from `--mkt-*` tokens / `@theme`; no new per-section hex. | grep |
| REQ-170-15 | "Six modes" and version **MUST** derive from `QUERY_MODES`/`getProductVersion()`; no magic literals re-added. | grep |

## C. Visual / UX quality

| ID | Requirement | Verify |
|----|-------------|--------|
| REQ-170-20 | The page **MUST** present ≥ 3 distinct focal moments (Hero, Benchmarks, QuickStart) with differentiated density — not 8 identical bands. | review + screenshots |
| REQ-170-21 | The hero **MUST** feature a real, current product screenshot from demo.edgequake.com, framed with elevation. | [04](./04-lens-database-data-layer.md) |
| REQ-170-22 | Cards **MUST** use the lighter two-layer elevation + hairline border (no `0 24px 64px`). | review |
| REQ-170-23 | Typography **MUST** use fluid `clamp()` scale; lede measure ≤ ~75 chars. | review |
| REQ-170-24 | Alternating section tone (base/alt) **MUST** be applied via the `Section` primitive. | review |
| REQ-170-25 | A quiet, factual proof strip (Apache 2.0 · stars · version · modes) **SHOULD** sit under the hero. | review |

## D. Accessibility (WCAG 2.2 AA)

| ID | Requirement | Verify |
|----|-------------|--------|
| REQ-170-30 | All text **MUST** meet AA contrast (4.5:1; 3:1 for large). | [11 G3](./11-metrics-and-acceptance.md) |
| REQ-170-31 | Interactive targets **MUST** meet WCAG 2.2 AA 2.5.8 (≥ 24×24 px, inline exempt); primary CTAs **SHOULD** reach ~44 px on mobile. | [11 G3](./11-metrics-and-acceptance.md) |
| REQ-170-32 | Exactly one `h1`; heading levels sequential; landmarks preserved. | axe/review |
| REQ-170-33 | Skip-link, `:focus-visible`, keyboard nav, ARIA tabs **MUST** be preserved. | keyboard test |
| REQ-170-34 | Decorative elements **MUST** be `aria-hidden`; images have meaningful `alt`. | review |

## E. Content integrity & SEO

| ID | Requirement | Verify |
|----|-------------|--------|
| REQ-170-40 | All claims/numbers **MUST** trace to repo/live product; no invented proof. | [07](./07-lens-marketing-growth.md) |
| REQ-170-41 | JSON-LD, canonical, OG/Twitter meta **MUST** be preserved (or improved). | diff review |
| REQ-170-42 | Build **MUST** succeed offline (no un-fallbacked build-time fetch). | [04](./04-lens-database-data-layer.md) |
| REQ-170-43 | `astro check` and `astro build` **MUST** pass. | [11 G5](./11-metrics-and-acceptance.md) |

## F. Traceability (requirement → lens → verification)

```
  REQ-170-01/02  ─ P2, Front §2 ──────────▶ G1 CLS gate
  REQ-170-03/04  ─ Data, UX §4 ───────────▶ G2 overflow/image gate
  REQ-170-10..15 ─ FullStack SOLID ───────▶ grep/review gate
  REQ-170-20..25 ─ UX, Front, Marketing ──▶ screenshot + review
  REQ-170-30..34 ─ UX a11y ───────────────▶ G3 a11y gate
  REQ-170-40..43 ─ Marketing, Data ───────▶ G5 build gate
```

Acceptance = **all MUST green**; SHOULD/MAY tracked but non-blocking. See
[11 — Metrics & Acceptance](./11-metrics-and-acceptance.md).
