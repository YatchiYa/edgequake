# 06 — Lens: Front Designer (visual system & no-CLS craft)

> Hub: [README](./README.md) · Prev: [05 UX/UI](./05-lens-ux-ui.md) · Next: [07 Marketing/Growth](./07-lens-marketing-growth.md)

The brief: *exceptional, not clunky, not AI slop*. Operationally that means a
**restrained, engineered** aesthetic where every visual decision has a reason
([P4](./00-why-first-principles.md)), realized on the existing `--mkt-*` token
system (extend, don't replace).

## 1. Design tokens (extend the existing system)

```
  EXISTING (keep):   --mkt-bg / -alt / -card · --mkt-text/-muted/-dim ·
                     --mkt-border · --mkt-accent-* · accent-50..900 scale
  ADD (reasoned):
    --mkt-ring            focus + hairline rings      (1px, accent@30%)
    --mkt-elevate         card shadow, LIGHTER        (0 1px 2px + 0 8px 24px @6%)
    --mkt-grid            faint background grid line   (for "engineered" texture)
    --space rhythm        section padding scale        (see §4)
    --mkt-accent-2        secondary hue for graph/data (teal/emerald, used sparingly)
```

Rule: **one** source of truth in [`global.css`](../../edgequake-website/src/styles/global.css).
No per-section hex values (kills ad-hoc drift).

## 2. Typography — and the CLS fix together

Keep the pairing (it's good): **Source Serif 4** (display) + **Source Sans 3**
(body) + **JetBrains Mono** (code). The defect is *how they load* (swap reflow,
[01 §2.1](./01-assessment-current-state.md)). Fix with **metric-matched fallbacks**:

```
  @font-face { font-family:"Source Serif 4 Fallback"; src: local("Georgia");
               size-adjust: …%; ascent-override: …%; descent-override: …%; line-gap-override:0%; }
  --font-display: "Source Serif 4", "Source Serif 4 Fallback", Georgia, serif;
  --font-sans:    "Source Sans 3",  "Source Sans 3 Fallback",  system-ui, sans-serif;
```

Fallback box metrics are tuned so the fallback line-box == the web-font line-box →
when the web font swaps in, **nothing reflows** → **CLS 0**. (Values derived/tuned
against the measured baseline and verified by re-measuring.)

Type scale (fluid, `clamp`) — avoids jumpy breakpoints:

```
  display-xl  clamp(2.5rem, 1.8rem + 3vw, 3.5rem)   hero h1
  h2          clamp(1.75rem, 1.4rem + 1.4vw, 2.25rem)
  lede        1.125rem / 1.7  (max-width ~36rem)
  body        1rem / 1.6
  mono        0.875rem
```

## 3. Color & tone

- Dark is the **primary** canvas (matches the product UI). Light fully supported.
- Single dominant accent (blue) for action; **secondary** (`--mkt-accent-2`,
  emerald/teal) *only* for the graph/data motif and success — never decorative.
- Replace heavy `0 24px 64px` card shadows with a **two-layer light elevation**
  (`0 1px 2px` + `0 8px 24px` at ~6%) + a **hairline border** → premium, not muddy
  (fixes C5).

## 4. Spatial rhythm (the thing that reads as "quality")

```
  Section vertical padding:  py-20 → standardized scale
     band-base:  clamp(4rem, 8vh, 6rem)
     band-focal: clamp(5rem, 10vh, 7.5rem)   (Hero, Benchmarks, QuickStart)
  Vertical rhythm inside: 8px base grid (mt-2/4/6/8/12/16 kept, used consistently)
  Max content width: 72rem (max-w-6xl) for text bands; hero/product may go wider
  Alternating tone: base / alt / base … gives scannable separation without borders
```

## 5. Motion — real, cheap, reduced-motion-safe (replaces dead FadeIn)

```
  Reveal primitive (CSS + ~1KB IntersectionObserver):
    default:        opacity:0; transform: translateY(12px)
    .is-visible:    opacity:1; transform: none        (transition 500ms ease-out)
    stagger:        --reveal-delay var applied per child (60–120ms steps)
    reduced-motion: opacity:1; transform:none; transition:none   (instant)
    no-JS:          content visible (enhancement only)
  ONLY animate opacity+transform (compositor) → never touches layout → CLS 0.
```

Micro-interactions (all transform/opacity/color, ≤200ms): button hover, card lift,
nav underline, tab indicator. No layout-affecting animation anywhere.

## 6. Hero composition (the focal #1)

```
  ┌───────────────────────────────── HERO (band-focal) ───────────────────────────────┐
  │  eyebrow: EDGEQUAKE                              ┌───────────────────────────────┐ │
  │  H1: Graph-RAG. Built to ship.                   │  ░ framed product shot ░       │ │
  │  lede: one binary · one Postgres · any LLM       │  graph.png (real demo)         │ │
  │  [ Get started → ]  [ Picture tour ] [ GitHub ★ ]│  browser chrome · soft shadow  │ │
  │  proof row: Apache 2.0 · 2.3k★ · v0.36 · 6 modes │  reserved aspect-ratio (no CLS)│ │
  │                                                  └───────────────────────────────┘ │
  │  faint engineered grid + single contained glow (overflow-clip, aria-hidden)        │
  └────────────────────────────────────────────────────────────────────────────────────┘
```

Product shot gets a lightweight **browser-frame** wrapper (dots + address bar hint)
and the two-layer elevation → feels like a real app, not a clip-art screenshot
([P5](./00-why-first-principles.md)). Intrinsic `width/height` set → zero image CLS.

## 7. Anti-slop checklist (self-audit before done)

- [ ] No gradient/shadow/animation without a named reason.
- [ ] No fake logos, testimonials, or metrics.
- [ ] No lorem/filler; every word is sourced ([04](./04-lens-database-data-layer.md)).
- [ ] Consistent spacing scale (no random `mt-7`, `p-5` one-offs).
- [ ] Icons from one registry, one stroke width.
- [ ] Dark **and** light both intentional (not an afterthought).
- [ ] CLS measured 0; no element escapes its section.

Realization details → [09 Implementation Plan](./09-implementation-plan.md).
