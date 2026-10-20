# 02 — Lens: Product Owner

> Hub: [README](./README.md) · Prev: [01 Assessment](./01-assessment-current-state.md) · Next: [03 Full-Stack](./03-lens-fullstack.md)

## 1. Outcome statement

> A skeptical engineer lands on edgequake.com, believes within 10 seconds that
> this is a *production-grade* Graph-RAG system, and leaves with a copy-pasted
> `quickstart.sh` command or a GitHub star — without encountering a single
> credibility leak (jump, jank, filler, or unverifiable claim).

## 2. Personas (who we are converting)

| Persona | Goal on the page | What wins them | What loses them |
|---------|------------------|----------------|-----------------|
| **Platform engineer** (primary) | "Can I self-host this safely?" | One binary + Postgres, SSO/RLS, explicit `migrate`, Apache 2.0 | vague "enterprise-ready" claims |
| **AI/ML engineer** | "Is retrieval actually better?" | Six modes, honest Acc vs LightRAG, graph shown | overclaimed benchmarks |
| **Eng manager / buyer** | "Is this a real project or a weekend repo?" | Stars, versioned crates, changelog, support path | dead links, stale version |
| **Returning dev** | "What's new; where are the docs?" | fast nav to Docs/Tutorial/What's-new | reflow, slow load |

## 3. Success criteria (measurable, owned here, verified in [11](./11-metrics-and-acceptance.md))

| ID | Criterion | Target | Why |
|----|-----------|--------|-----|
| PO-1 | Layout stability | **CLS = 0** on home (all 4 view/theme combos) | [P2](./00-why-first-principles.md) credibility |
| PO-2 | No unintended horizontal overflow | 0 px body overflow; no element escapes its section | craft |
| PO-3 | Primary CTA reachable above the fold | `quickstart` + GitHub visible < 1 viewport on desktop | conversion |
| PO-4 | Every claim is sourced | 100% of numbers/labels trace to repo/live product | [P1](./00-why-first-principles.md) |
| PO-5 | A11y preserved/improved | AA contrast; 44px tap targets; keyboard + reduced-motion OK | inclusive + trust |
| PO-6 | No regression in SEO/meta | JSON-LD, OG, canonical unchanged or improved | organic reach |
| PO-7 | Build is green | `astro check` + `astro build` pass | ship-ability |

## 4. Scope & prioritization (MoSCoW)

**Must**
- Eliminate font-swap CLS (metric-matched fallbacks) — fixes PO-1.
- Replace dead `FadeIn` with a real, reduced-motion-safe `Reveal` — fixes C2.
- Extract shared `SectionHeader` + `eyebrow` + icon registry — fixes C6/C7 (DRY).
- Reshape rhythm: product-forward hero, alternating section density — fixes C1/C4.
- Contain the hero glow in-section; stop relying on body clip — fixes C9.

**Should**
- Elevate card system (lighter shadow, hairline border, hover lift) — C5.
- Add a subtle, honest "proof band" (stars, version, Apache 2.0, modes) under hero.
- Refine type scale + measure (line length) for readability.

**Could**
- Secondary accent hue for the graph/data story (used sparingly).
- Animated-but-cheap graph motif in hero background (reduced-motion-safe).

**Won't (this spec)**
- Rewrite Starlight docs theme, tutorial pages, or the product WebUI.
- Add fake testimonials/logos or any unverifiable social proof.
- Introduce a heavy JS framework or client-side router.

## 5. Non-goals / guardrails

- **No new runtime dependencies** unless justified; the site stays static Astro.
- **No content invention** — copy edits must remain truthful and sourced.
- **No dark-pattern CTAs** — honesty is the brand ([07](./07-lens-marketing-growth.md)).

## 6. Definition of Done

All of PO-1…PO-7 green in [11](./11-metrics-and-acceptance.md), spec requirements
in [08](./08-specification.md) satisfied, edge cases in
[10](./10-edge-cases-and-testing.md) covered, and before/after screenshots show a
clearly elevated, consistent page with **no layout shift**.
