# 03 — Implementation Roadmap (SPEC-099)

## Waves

| Wave | Focus | DoD |
|------|-------|-----|
| **W0** | Spec package + evidence | This folder complete; findings/laws locked; cross-ref matrix filled |
| **W1** | Unify status SSOT | All normalize/display/terminal/processing/rank imports resolve to `status-domain.ts`; `status-badge.tsx` is presentation map only; domain unit tests cover former badge edge cases; no dual export |
| **W2** | StatusCell fence presentation | One Status cell; Ready not a peer success pill; `query_ready` still in DOM/a11y (`data-query-ready`); `spec091` green; `spec099-status-cell-fence` green |
| **W3** | Upload collapse + toast demotion | Dropzone `data-collapsed=true` when feedback zone has live work; drop still works; toast suppressed when zone lists same session; `spec048` + `spec350` + `spec099-upload-collapse` + `spec099-toast-demotion` green |
| **W4** | Feedback zone density | Compact run cards; with ≥6 concurrent runs, zone ≤35vh scroll and ≥2 table rows visible on 900px height; `spec099-feedback-viewport` + `spec086` green |
| **W5** | Destructive hierarchy + selection mode | Clear All not peer to Refresh; typed confirm retained; selection mode replaces toolbar row; `spec099-clear-all-demoted` green |
| **W6** | Shell SRP split | `DocumentManager` thin shell; zone/table/upload controllers; single pipeline UI resolve; row actions via context; no behavior regress on 048/050/086/091/098 |
| **W7** | Scale + filter honesty | Overflow / N of M when capped; header ↔ chip ↔ rows one view-model; NEW demoted or removed; Cost default-hidden or secondary; `spec099-filter-count-parity` green |
| **W8** | E2E matrix + CI | All `spec099-*` Playwright + units wired; non-regression suite green; ux-ui-audit selectors fixed (F-099-13) |

## Wave dependency graph

```ascii
 W0 Spec
   → W1 Status domain SSOT
     → W2 StatusCell presentation
       → W3 Upload collapse + toast
         → W4 Zone density
           → W5 Destructive + selection
             → W6 Shell SRP
               → W7 Scale + filter honesty
                 → W8 Gates + CI
```

W1 is a hard prerequisite for W2/W7 (display status + failed highlight). W3/W4 may proceed in parallel after W1 if StatusCell API is stable.

## Primary files by wave

| Wave | Primary files |
|------|---------------|
| W1 | `src/lib/documents/status-domain.ts`, `src/components/documents/status-badge.tsx`, `src/lib/utils/document-status.ts`, callers under `src/hooks/`, `src/lib/pipeline/`, tests |
| W2 | `src/components/documents/enhanced-status-badge.tsx`, a11y labels, Playwright `spec099-status-cell-fence` |
| W3 | `document-dropzone.tsx`, `document-toolbar-section.tsx`, `use-file-upload.ts` |
| W4 | `active-runs-panel.tsx`, `ingestion-run-card.tsx`, `server-stage-stepper.tsx` / `phase-strip.tsx` |
| W5 | `clear-documents-dialog.tsx`, toolbar/header in manager or toolbar section |
| W6 | `document-manager.tsx` extract → shell + controllers |
| W7 | `use-document-queries.ts`, `use-document-filtering.ts`, table column defaults |
| W8 | `e2e/spec099-*.spec.ts`, `e2e/ux-ui-audit.spec.ts`, CI workflow if needed |

## Exit criteria

- [x] Spec docs under `specs/099-ux-ui-improvement/` (W0)
- [x] Single status domain import path; badge has no domain helper exports
- [x] StatusCell composite; fence semantics preserved; no peer dual green pills
- [x] Busy: upload collapsed + toast demoted; idle: expandable dropzone (always-on full-width band)
- [x] Feedback zone denser; viewport budget holds with multi-run
- [x] Clear All demoted; typed confirm retained
- [x] DocumentManager SRP split; prop drilling reduced (`useDocumentsInventory` + `useLiveWorkControllers` + actions context)
- [x] Scale honesty (N of M / overflow) + filter count parity
- [x] All SPEC-099 Playwright/unit gates green (wired in `e2e-quality-gates.yml`)
- [x] Inventory scroll stays internal; chrome + dropzone pinned (`spec099-documents-scroll-layout`)
- [x] Refresh CLS: feedback-zone reservation + soft-refresh placeholder (`spec099-layout-stability`)
- [ ] Non-regression: 048 / 050 / 086 / 091 / 098 / 350 green (run in CI / local verification)

## Out of scope (explicit)

- Backend pagination API redesign beyond UI honesty affordances (may stub “showing N” until GH-319 server work).
- Full WebUI redesign of Dashboard / Query / KG.
- Changing `query_ready` semantics or delete admit dual-write (SPEC-091 / SPEC-098).
