# 08 — E2E Test Matrix

## Unfakable rules

Assert DOM attributes and URL — never “looks synced” screenshots alone.

| ID | Scenario | Assert |
|----|----------|--------|
| E-143-01 | Open side-by-side fixture with markers | `pdf-page-indicator[data-page="1"]` and `[data-eq-page="1"]` exist |
| E-143-02 | Wheel / scroll PDF stack to page 2+ | indicator `data-page` updates; URL has `page=` |
| E-143-03 | Sync ON: PDF page → MD | `#eq-md-page-N` in viewport (or scrollTop moved toward it) |
| E-143-04 | Sync ON: scroll MD to page-4 section | `pdf-page-indicator[data-page="4"]` |
| E-143-05 | Sync OFF: change PDF page | markdown `scrollTop` unchanged (within epsilon) |
| E-143-06 | Toolbar next / PageDown | same as E-143-02 |
| E-143-07 | Deeplink `?page=4` | both panes on 4 (regression SPEC-142) |
| E-143-08 | No markers fixture | sync toggle `disabled`; PDF stack still navigable |

## Unit companions

| ID | Target |
|----|--------|
| U-143-01 | `injectPageAnchors` round-trip |
| U-143-02 | Controller lock: pdf driver ignores md until settle |
| U-143-03 | `listPageMarkers` ignores MM fence noise |

## Fixture strategy

Reuse blank multi-page PDF + markdown with `<!-- edgequake-page:1..N -->`
(same pattern as `spec142-precise-links.spec.ts` / `blank-pdf.ts`).

## File

`edgequake_webui/e2e/spec143-pdf-markdown-sync.spec.ts`

## Cross-refs

- Acceptance: [10-acceptance.md](10-acceptance.md)
- Edge cases: [09-edge-cases.md](09-edge-cases.md)
