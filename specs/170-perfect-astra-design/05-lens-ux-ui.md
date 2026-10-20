# 05 — Lens: UX / UI Designer

> Hub: [README](./README.md) · Prev: [04 Data Layer](./04-lens-database-data-layer.md) · Next: [06 Front Design](./06-lens-front-design.md)

## 1. Information architecture — the scroll as a narrative

The page must answer the visitor's questions **in the order they ask them**. Today
the order is roughly right but every band has equal weight, so the narrative is
flat. We keep the sequence, reshape the emphasis:

```
  Q the visitor is asking          Band                 Emphasis (new)
  ────────────────────────         ────                 ──────────────
  "What is this, in one line?"  →  HERO               →  ★★★ focal, product-forward
  "Prove it's real"             →  PROOF STRIP        →  ★   quiet, factual (NEW, cheap)
  "Why not just vector search?" →  PROBLEM            →  ★★  tension
  "What do I actually get?"     →  SOLUTION           →  ★★  relief / capability
  "How does it work?"           →  ARCHITECTURE       →  ★★  one diagram, legible
  "Is it actually good?"        →  BENCHMARKS         →  ★★★ honest numbers (focal #2)
  "How do I start?"             →  QUICKSTART         →  ★★★ copy-paste (focal #3)
  "What's the surface area?"    →  ECOSYSTEM          →  ★   scannable
  "I'm a buyer—talk to me"      →  ENTERPRISE CTA     →  ★★  conversion close
```

Three focal moments (Hero, Benchmarks, QuickStart) give the eye somewhere to land
and rest — fixing the "wall of equal bands" critique C1 ([01](./01-assessment-current-state.md)).

## 2. Interaction & state inventory (every state designed, not accidental)

| Element | States to design | Current gap |
|---------|------------------|-------------|
| Buttons (primary/outline) | rest · hover · focus-visible · active | OK; unify via `Button` primitive |
| Nav link | rest · hover · current-page · focus | OK |
| Theme toggle | dark→light, icon swap, `theme-color` meta | OK; keep |
| QuickStart tabs | selected · hover · focus · keyboard (←/→/Home/End) | OK (good ARIA); keep |
| Cards | rest · hover (lift) · focus-within (if link) | shadow too heavy (C5) |
| Reveal-on-scroll | hidden→shown · reduced-motion · no-JS | **missing** (dead FadeIn, C2) |
| Images | reserved box before load | ensure every `<img>` has w/h |
| `<details>` micro-bench | collapsed · expanded · focus | OK |

## 3. Accessibility contract (WCAG 2.2 AA)

Preserve what exists (skip-link, `:focus-visible`, `prefers-reduced-motion`,
landmarks, ARIA tabs) and add:

- **Contrast** — all text ≥ 4.5:1 (≥ 3:1 for ≥ 24px/bold). Audited in
  [11](./11-metrics-and-acceptance.md); watch `mkt-text-dim` on `mkt-bg-alt`.
- **Tap targets** — interactive elements ≥ 44×44 px on mobile (measured).
- **Motion** — reveals are opacity/transform only and fully disabled under
  reduced-motion; no autoplaying looping motion that can't be paused.
- **Headings** — one `h1` (Hero), sequential `h2` per band, `h3` within. No skips.
- **Images** — meaningful `alt`; decorative glows are `aria-hidden`.
- **Keyboard** — full path: skip-link → nav → hero CTAs → tabs → footer, visible focus.

## 4. Responsive system

```
  Breakpoints (Tailwind):  base <640 · sm 640 · md 768 · lg 1024 · xl 1280
  Hero:    lg → 2-col (copy | product)         base → 1-col, product below copy
  Grids:   lg → 3-up · sm → 2-up · base → 1-up (cards reflow, never overflow)
  Code:    base → internal x-scroll inside a bordered surface (not body scroll)
  Nav:     md → inline · base → hamburger sheet (exists; keep, verify focus trap)
```

**No horizontal page scroll at any width** — guaranteed by containing decorative
glows inside their section (`overflow-clip`) rather than relying on the body clip
(fixes C9). Verified by the overflow metric in [11](./11-metrics-and-acceptance.md).

## 5. Microcopy principles (UX writing)

- Lead with the **verb/outcome**, not the feature name.
- Numbers over adjectives ("13,200 entities" > "powerful extraction").
- Keep the honest hedge in Benchmarks — it builds more trust than a bold claim.
- Line length 50–75 chars for lede paragraphs (readability) — enforced via `max-w-*`.

Cross-ref: visual realization of these decisions → [06 Front Design](./06-lens-front-design.md).
