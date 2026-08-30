# 11 — Honest Assessment

## What this pack fixes

- Answer-inline citations become **locator-verified** (document + page from storage).
- Closes the deferred SPEC-047 L-B1 **answer-inline** half for chunk cites.
- Fixes UUID chunk → wrong document id mapper bug.
- **P0.5:** Uncatalogued numeric prose pages (`page 999`) are scrubbed; multi-doc chips
  carry a short stem; rewrite counts are observed (LAW-142-11..13).

## What this pack does **not** fix

| Gap | Why |
|-----|-----|
| Claim ↔ chunk faithfulness (NLI) | P1 latency/cost; LAW-142-10 |
| Non-numeric page prose (“the fifth folio”) | No numeral to match; residual until P1 / scrub expand |
| Entity cite → page without chunk hop | SPEC-047 entity pages still deferred |
| Pixel-perfect bbox highlight | Optional; line/highlight params already exist |
| Live-LLM citation quality | Unfakable tests use mock/scripted answers |

## Residual risk

Strong models may still invent **non-numeric** page prose or omit `[N]` entirely.
Users clicking **links** are safe; locator numerals in prose are scrubbed when
uncatalogued. Claim↔chunk mismatch (correct page, wrong support) needs P1 NLI
bound to the **cited chunk**, not max-over-retrieval.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
