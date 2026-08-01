# 04 — E2E Test Matrix (SPEC-099)

All paths relative to `edgequake_webui/` unless noted.

## Non-regression suite (must stay green)

| Gate | Command / file | Protects |
|------|----------------|----------|
| SPEC-048 | `pnpm exec playwright test e2e/spec048-ingestion-progress.spec.ts` | Active runs, quiet/`data-quiet`, phase parity |
| SPEC-050 | `pnpm exec playwright test e2e/spec050-delete-feedback-zone.spec.ts` | Delete in feedback zone |
| SPEC-086 | `pnpm exec playwright test e2e/spec086-ingestion-ux.spec.ts` | Dual-run, cancel, Needs attention |
| SPEC-091 | `pnpm exec playwright test e2e/spec091-ingestion-surface.spec.ts` | Fence Ready vs Indexed **truth** |
| SPEC-098 | `pnpm exec playwright test e2e/spec098-bulk-delete-honesty.spec.ts` | Mid-delete ≠ Completed/Ready |
| SPEC-350 | `pnpm exec playwright test e2e/spec350-bulk-upload-webui.spec.ts` | Multi-file drop → table |
| Domain unit | `bun test src/lib/documents/__tests__/status-domain.test.ts` | Domain predicates |
| Merge unit | `bun test src/lib/documents/__tests__/merge-monotonic-list.test.ts` | List merge |
| Delete session unit | `bun test src/lib/documents/__tests__/deletion-session.test.ts` | Pins/sessions |

## New SPEC-099 gates

| Gate ID | Wave | Type | Assert | Finding |
|---------|------|------|--------|---------|
| `spec099-status-domain-single-import` | W1 | unit / lint | Badge module does not export `normalizeStatus` / `getDocumentDisplayStatus` / `isProcessingStatus` / `isTerminalStatus`; domain is sole SSOT | F-099-01 |
| `spec099-status-cell-fence` | W2 | Playwright | Completed terminal shows **one** Status cell; Ready is not a second peer success pill; `data-query-ready` still present for Ready/Indexed; a11y name includes fence | F-099-02 |
| `spec099-upload-collapse` | W3 | Playwright | When Active runs open, dropzone has `data-collapsed="true"` (or equivalent); file drop / click upload still works | F-099-04 |
| `spec099-toast-demotion` | W3 | Playwright | While feedback zone lists upload session files, no persistent “Uploading N file(s)…” toast competing as third SSOT | F-099-03 |
| `spec099-feedback-viewport` | W4 | Playwright | With N≥4 seeded active/queued runs, feedback zone max-height ≤35vh (or scroll container); table section still visible | F-099-06 |
| `spec099-live-row-no-stage-subtitle` | W4 | Playwright | Document with Active runs card has no `spec048-row-stage` (or successor) subtitle in table row | F-099-07 |
| `spec099-clear-all-demoted` | W5 | Playwright | Clear All not adjacent primary peer to Refresh in header; reachable via overflow / danger control; typed confirm still required | F-099-05 |
| `spec099-selection-toolbar` | W5 | Playwright | With ≥1 row selected, selection actions replace (not stack under) the primary toolbar row | F-099-16 |
| `spec099-filter-count-parity` | W7 | Playwright | Header document count, status filter chip total, and visible row count agree for the active filter | F-099-10 |
| `spec099-scale-overflow` | W7 | Playwright or unit | When fetch capped / total > page size, UI shows overflow or “showing N of M” — not silent full-corpus implication | F-099-09 |
| `spec099-banner-demote` | W4/W6 | Playwright | Non-stuck pipeline banner hidden when feedback zone open | F-099-14 |
| `spec099-ux-audit-dropzone` | W8 | Playwright | `ux-ui-audit` (or successor) locates `data-testid="document-dropzone"` | F-099-13 |
| `spec099-documents-scroll-layout` | W8+ | Playwright | After scrolling `documents-table-scroll`, dropzone still in viewport; `window.scrollY===0`; inventory fills shell | F-099-16 · EC-099-16 |
| `spec099-layout-stability` | W8+ | Playwright | Cold load with live-work hint: feedback skeleton → Active runs; inventory Y stable; soft refresh no bounce; CLS &lt; 0.15 | F-099-17 |

## Suggested file layout

```text
edgequake_webui/e2e/
  spec099-status-cell-fence.spec.ts
  spec099-upload-collapse.spec.ts
  spec099-toast-demotion.spec.ts
  spec099-feedback-viewport.spec.ts
  spec099-live-row-no-stage-subtitle.spec.ts
  spec099-clear-all-demoted.spec.ts
  spec099-selection-toolbar.spec.ts
  spec099-filter-count-parity.spec.ts
  spec099-scale-overflow.spec.ts   # optional fold into filter-count
  spec099-banner-demote.spec.ts

edgequake_webui/src/lib/documents/__tests__/
  status-domain.test.ts            # extended W1
  status-badge-no-domain-export.test.ts  # or eslint rule
```

## CI wiring (W8)

Prefer extending the existing WebUI / e2e quality-gate workflow that already runs SPEC-048/086/091/098 Playwright. Add `spec099-*.spec.ts` to the same job; do not invent a parallel unwatched suite (LAW-099-10).

## Manual exploratory checklist (not a substitute for gates)

- [ ] Idle laptop 1440×900: table ≥60% content height after W3/W4
- [ ] Upload 7 PDFs: one narrative surface, inventory still scannable
- [ ] Mid-delete still never shows Completed/Ready (098)
- [ ] Indexed-not-queryable fence still distinguishable from Ready
- [ ] Keyboard: dropzone collapse still activatable; Clear All still confirmable
