# 12 — Design Plan (frontend-design skill applied)

> Hub: [README](./README.md) · Applies the installed
> [frontend-design skill](../../.github/skills/frontend-design/SKILL.md) to the
> homepage. Supersedes the generic treatment critiqued in
> [01](./01-assessment-current-state.md).

## Subject (grounding)

EdgeQuake turns documents into a **knowledge graph** in PostgreSQL and answers
with sources, on self-hostable Rust. Audience: skeptical platform / AI engineers.
Job: prove it is production-grade and earn a `quickstart.sh` run or a star. The
single most characteristic thing in its world is the **graph itself** — nodes and
edges, a network topology. That is where boldness is spent; everything else stays
quiet and engineered.

## Honest audit — our old page had the skill's named "tells"

The skill lists AI-generated defaults. Our prior redesign contained **six** of them:

| Tell (from skill) | Where it was | Fix |
|-------------------|--------------|-----|
| Tracked-out ALL-CAPS eyebrow above every heading | `SectionHeader eyebrow` on all bands | **Removed** — eyebrows off by default |
| Accenting one phrase of a headline in another color | Hero "**Built to ship.**" in blue | **Removed** — monochrome headline, type does the work |
| `→` appended to button text | Hero/Enterprise CTAs `iconRight` | **Removed** |
| Meta strings joined with middle dots (`A · B · C`) | hero caption, proof copy | **Rewritten** as plain sentences |
| SaaS-card kit: one radius + one soft shadow on everything | `.mkt-card` everywhere | **Differentiated** by hierarchy; Problem de-carded |
| High-contrast **serif display** (generic pairing) | Source Serif 4 + Source Sans 3 | **Replaced** (see Type) — *this is the font the client hates* |
| Fade-and-slide-up on every section/card | `Reveal` on all bands + staggered cards | **Reduced** to one orchestrated hero moment |

## Token system

### Type (deliberate, subject-grounded; not our usual defaults)
- **Display → Space Grotesk.** A geometric technical grotesque: engineered,
  slightly mechanical, strong numerals — reads like systems software, not a
  magazine. Carries headlines and the benchmark figures. `size-adjust 103.77%`
  (measured vs Arial) for zero font-swap CLS.
- **Body → IBM Plex Sans.** Purpose-built by IBM for technical products;
  infrastructure heritage, highly legible, clearly distinct from Space Grotesk.
  `size-adjust 101.23%`.
- **Mono → JetBrains Mono.** Code blocks and package identifiers only.
- Type scale (Elements of Typographic Style ratios, fluid `clamp`): display
  `clamp(2.6rem,1.9rem+3vw,3.9rem)` / h2 `clamp(1.7rem,1.35rem+1.4vw,2.2rem)` /
  lede `1.15rem/1.65` / body `1rem/1.6`. Body measure < 72ch.

### Color (cool engineering ink + the graph's own energy)
```
  --ink-900  #0A0C12   page (deep cool ink — a blueprint, not zinc-black)
  --ink-800  #12151D   raised surface
  --ink-700  #1B1F2A   hairline borders / wells
  --paper    #FBFCFE   light base
  --signal   #4F6BFF   primary action (electric indigo — EdgeQuake "edge")
  --graph    #34D399   the knowledge-graph accent (nodes); used sparingly
  --text     #EAECEF / --muted #9BA2AE / --dim #747C8A (all AA-verified)
```
Boldness lives in the graph image + one `--signal` CTA. No gradient decoration.

### Layout (engineered, not all-centered)
```
  HERO        [ headline · subhead · 2 CTAs · quiet proof ] | [ GRAPH — the bold thing ]
  PROBLEM     left header + two-column reasoned rows (divided, NOT 6 identical cards)
  SOLUTION    left header + capability list (link rows)
  SHOWCASE    the graph, large + 4 inline figures + 2 supporting product shots
  ARCHITECTURE legible pipeline diagram (a real sequence)
  BENCHMARKS  left header + large honest numbers + comparison table
  QUICKSTART  tabs + copy-paste commands
  ECOSYSTEM   surfaces + compact crate grid
  ENTERPRISE  quiet close + one primary CTA
```
Section headers **left-aligned** (engineered feel), content follows a 12-col sense
of order. Alternating ink tone for separation instead of boxes-everywhere.

### Principles (what makes this page this page)
1. The graph is the hero and the only loud element.
2. Engineered typography (grotesque + Plex) over editorial serif.
3. Honesty as design: real numbers, no overclaim, plain sentences (no `·`/`→` chrome).
4. Motion is one orchestrated hero reveal; interaction motion only elsewhere.
5. Quality floor unannounced: AA contrast, keyboard focus, reduced-motion, CLS 0.

## Uniqueness review (per skill, before building)

- Would a generic prompt reach Space Grotesk + IBM Plex Sans on cool ink with an
  emerald graph accent? No — the default is Inter/Source + blue + cards. This is a
  deliberate, subject-grounded deviation. ✔
- Any remaining default spent on a "free" axis? Dark base is **not** a free default
  here — it matches the product UI (the demo is dark), so it's a grounded choice,
  not the "near-black + one acid accent" cliché; our accent is a restrained signal
  indigo + the graph's own emerald, taken from the real product. ✔
- Boldness concentrated in one place (graph)? ✔

Build follows this plan; critique via Playwright screenshots per band
([11](./11-metrics-and-acceptance.md), [`e2e/inspect.mjs`](./e2e/inspect.mjs)).
