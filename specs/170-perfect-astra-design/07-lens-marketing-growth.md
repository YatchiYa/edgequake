# 07 — Lens: Marketing & Growth

> Hub: [README](./README.md) · Prev: [06 Front Design](./06-lens-front-design.md) · Next: [08 Specification](./08-specification.md)

## 1. Positioning (one sentence, defended)

> **EdgeQuake is production Graph-RAG you can self-host: one Rust binary, one
> Postgres, any LLM — with the knowledge graph, auth, and honest benchmarks a
> real deployment needs.**

Differentiators that are *true* and *checkable* (so they survive a skeptical read):

| Claim | Proof on page | Source |
|-------|---------------|--------|
| Production, not prototype | SSO/RLS, explicit `migrate`, GHCR multi-arch images | Ecosystem/Enterprise copy |
| Rust performance | 82 ms warm p50; ~10× ingest (historical, disclosed) | Benchmarks |
| Honest quality | Acc tie vs LightRAG, "no win when CI includes 0" | Benchmarks |
| Any LLM, local-first | OpenAI/Anthropic/Mistral/Gemini/Ollama, encrypted keys | Solution |
| Real product | live graph with 13.2k entities (demo captures) | [04](./04-lens-database-data-layer.md) |
| No lock-in | Apache 2.0, on-prem, no cloud-embedding requirement | Enterprise trust row |

## 2. The narrative arc (AIDA, engineer-grade)

```
  ATTENTION  Hero: crisp one-liner + real product graph (show, don't tell)
  INTEREST   Problem: name their pain precisely (6 concrete failures of vector RAG)
  DESIRE     Solution + Benchmarks: capability + HONEST numbers (trust compounds)
  ACTION     QuickStart (copy-paste) → Enterprise CTA (talk to team)
```

Honesty *is* the growth strategy for this audience: the Benchmarks section that
refuses to overclaim converts better than a fabricated "10× better" because the
reader can verify it and therefore trusts everything else.

## 3. Conversion design

**Primary CTA:** `Get started` → `#quickstart` (copy-paste `quickstart.sh`).
**Secondary:** GitHub (star = low-friction micro-conversion), Picture tour, Demo.
**Buyer path:** Enterprise CTA → `/contact/`.

Rules:
- One unmistakable primary action per focal band; never two equal-weight CTAs.
- The command is the CTA — show the actual `curl … | sh` early (reduces "how hard
  is this?" anxiety).
- Social proof is **only** real: GitHub stars (live, floored), version, Apache 2.0.
  No invented customer logos ([02 Won't](./02-lens-product-owner.md)).

## 4. Messaging hierarchy (what to cut / amplify)

| Keep & amplify | Trim / de-emphasize |
|----------------|---------------------|
| "Built to ship" tagline (strong, true) | redundant adjectives in card copy |
| Honest benchmark framing | long historical micro-bench (keep in `<details>`) |
| Six retrieval modes (derive from `QUERY_MODES`) | repeating "v0.36.0" on every crate card |
| Self-host / no lock-in | generic "enterprise-ready" phrasing |

## 5. SEO / shareability (don't regress — [PO-6](./02-lens-product-owner.md))

- Preserve JSON-LD `@graph` (Organization/WebSite/SoftwareApplication/WebPage),
  canonical, OG/Twitter tags in
  [`MarketingLayout.astro`](../../edgequake-website/src/layouts/MarketingLayout.astro).
- Keep `og-default.png` (1200×630). If hero art changes materially, refresh OG to
  match (optional, Could).
- Semantic headings + descriptive `alt` improve both a11y and SEO.
- Fast, stable (CLS 0) pages rank and share better — growth and craft align.

## 6. Growth instrumentation (lightweight, no new deps)

- Stable `id`s/anchors (`#quickstart`, `#benchmarks`) for shareable deep links.
- `data-*` hooks on primary CTAs so analytics can be added later without redesign.
- No tracker added in this spec (privacy-respecting default; can be a later toggle).

## 7. Success = behavior change

The marketing win condition mirrors [02 PO](./02-lens-product-owner.md): the page
earns the copy-paste or the star by being *demonstrably* crafted and *verifiably*
honest. Every requirement in [08](./08-specification.md) ladders up to that.
