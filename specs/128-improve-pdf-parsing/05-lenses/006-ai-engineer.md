# Lens 006 — AI Engineer

## Stake

L3 is a **semantic oracle** on already-proposed crops, not a region detector. Pass-1 must be cheap, concurrent, and budgeted. Pass-2 specialize must not leak into overlay taxonomy (overlay classes come from L0/L1/L2 + derived columns).

## Control loop

```ascii
  Geometry + layout propose PNG paths
       → Pass-1 JSON { kind, is_figure, confidence }
       → kept = is_figure
       → Pass-2 description for kept only (optional MD enrich; v1 may still ignore at assemble)
       → PRUNE figure_map to kept paths
```

Unknown kind → `Other` → **keep** (fail-open), same as today.

## Prompts

Extend `FIGURE_FILTER_PASS1_SYSTEM` with discard kinds: `stamp`, `signature`, `scan_artefact`, `watermark` (all `is_figure: false`). Do not ask the VLM to emit page-wide layout classes — that is L2.

## Budget

- `figure_filter_concurrency` default 4 (`buffer_unordered`)
- `max_figure_vlm_per_page` default 12 — drop lowest-area candidates **after** geometry, **before** Pass-1
- Geometry + L2 first so VLM never sees full-page dumps or 1px spacers

## Caching

Respect SPEC-103 LLM cache flags if Pass-1 messages are cacheable; crop images differ per asset so cache hit rate is low. Do not invent a second cache. Prompt-cache (SPEC-126) may apply to the static Pass-1 system prompt.

## Observability

Span `figure.filter.pass1` / `pass2` with GenAI attrs when the provider returns usage. Counter `vlm_discarded_by_kind`. Never put crop PNG bytes in span I/O (PII / size).

## Eval

G-industrial: logo/stamp discarded; real diagram kept. MockProvider contract first; live LLM gated on API key (existing pattern in `contract_spec049_figure_filter.rs`).

## Cross-refs

- Filter module: `edgequake-pdf/src/figure_filter.rs`
- Plan WP-0/1/3/4: [../07-implementation-plan.md](../07-implementation-plan.md)
- Taxonomy vs kinds: [../13-layout-taxonomy.md](../13-layout-taxonomy.md)
