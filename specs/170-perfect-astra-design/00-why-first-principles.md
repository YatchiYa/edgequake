# 00 — WHY & First Principles

> Hub: [README](./README.md) · Next: [01 Current-State Assessment](./01-assessment-current-state.md)

## 1. The WHY (start here)

A developer-tools homepage has exactly **one job**: convert a skeptical engineer
from *"what is this?"* to *"I'll run `quickstart.sh`"* in under ~30 seconds, without
lying to them. EdgeQuake's current site is competent but **generic** — it reads
like a template. For an infrastructure product whose entire pitch is *"built to
ship, production-grade, not a prototype glue stack,"* a templated site is a
**credibility leak**: the medium contradicts the message.

```
   The contradiction we are fixing
   ┌──────────────────────────────────────────────────────────┐
   │  MESSAGE:  "Production-grade. Rust. Built to ship."        │
   │  MEDIUM:   generic template · no motion · flat rhythm      │
   │            8 identical centered sections · dead FadeIn     │
   │  RESULT:   the site undersells a genuinely strong product  │
   └──────────────────────────────────────────────────────────┘
```

So the WHY is not "make it prettier." It is: **make the craft of the page prove
the craft of the product**, and remove every friction/credibility leak between a
visitor and `quickstart.sh`.

## 2. First principles

We derive the design from irreducible truths rather than from "what other SaaS
sites do."

### P1 — The audience is adversarial-skeptical engineers
They distrust marketing. They trust: real numbers, real screenshots, copy-paste
commands, source links, and *restraint*. → The page must look **engineered**, not
**advertised**. Honesty is a feature (the current Benchmarks section already says
"we do **not** claim a win when the CI includes zero" — that tone is an asset to
amplify, not dilute).

### P2 — Performance IS the first design impression
For this audience, a layout that jumps while loading (CLS) or a heavy, janky page
*is* the product's first bug. The homepage's own Core Web Vitals are a live demo of
"we care about the details." → **CLS = 0** is a design requirement, not a QA
afterthought. (Baseline measured 0.023 in dark mode, caused by web-font swap — see
[01](./01-assessment-current-state.md).)

### P3 — Attention is spent top-down and must be *shaped*
A page of eight equal-weight, equal-rhythm, center-aligned sections has **no
focal hierarchy** — everything is a heading + subhead + grid, so nothing leads.
First principles of visual perception say the eye needs **contrast, rhythm
variation, and a clear path**. → Introduce deliberate rhythm: a commanding hero,
alternating alignment/density, one "hero moment" per scroll-depth.

### P4 — Every pixel must be earned by a reason
"No AI slop" operationalized: a gradient, a shadow, an animation, or a word exists
only if it serves comprehension, trust, or conversion. If we cannot name the reason,
we delete it. → This kills decorative noise and forces intentional craft.

### P5 — The product is the hero, shown truthfully
The strongest asset EdgeQuake has is the **actual product** (knowledge graph,
six retrieval modes, grounded answers). → Real, current screenshots from
demo.edgequake.com, framed to feel tangible — not a tiny static thumbnail.

### P6 — One source of truth, composed
DRY/SOLID isn't only backend hygiene; a design system with one token source, one
section-header primitive, and data-driven content is what keeps the site
**consistent** (which *is* the premium feel) and **maintainable**. → See
[03 Full-Stack](./03-lens-fullstack.md) and [09 Plan](./09-implementation-plan.md).

## 3. From principles → design intents

```
 P1 skeptical engineers ─┐
 P5 product is hero ─────┼─▶ INTENT A: "Show, don't sell" — product-forward hero,
 P4 earn every pixel ────┘             real numbers, copy-paste first.

 P2 performance = craft ─┐
 P6 one source of truth ─┼─▶ INTENT B: "Engineered feel" — CLS=0, self-hosted/
                         │             metric-matched fonts, token system, 60fps
                         │             reduced-motion-safe reveals.
 P3 shape attention ─────┴─▶ INTENT C: "A path, not a wall" — deliberate rhythm,
                                        focal hierarchy, one idea per band.
```

Each intent maps forward to concrete requirements in
[08 Specification](./08-specification.md) and is validated by gates in
[11 Metrics & Acceptance](./11-metrics-and-acceptance.md).

## 4. What "exceptional, not clunky, not AI slop" means here

| Quality | Clunky / slop (reject) | Exceptional (target) |
|--------|------------------------|----------------------|
| Rhythm | 8 identical centered bands | Shaped hierarchy, one focal moment per band |
| Motion | Dead `FadeIn` (no-op) | Real, GPU-cheap, reduced-motion-safe reveals |
| Type | Render-blocking Google Fonts → swap/CLS | Metric-matched fallback, preloaded, **0 reflow** |
| Trust | Vague adjectives | Real Acc numbers, honest ties, source links |
| Imagery | Tiny static thumbnail | Framed, current product shots from the demo |
| Copy | Filler, buzzwords | Specific, engineer-grade, verifiable |
| Consistency | Ad-hoc Tailwind per section | Token system + shared primitives (DRY/SOLID) |

Proceed to [01 — Current-State Assessment](./01-assessment-current-state.md) for the
evidence base.
