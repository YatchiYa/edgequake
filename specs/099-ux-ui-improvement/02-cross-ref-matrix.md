# 02 — Cross-Ref Matrix (SPEC-099)

Paths are under `edgequake_webui/` unless noted.

| Finding | Code | Test | Law |
|---------|------|------|-----|
| F-099-01 | `src/lib/documents/status-domain.ts` SSOT; badge presentation-only | unit: `status-domain.test.ts` + `status-badge-no-domain-export.test.ts` | LAW-099-1 |
| F-099-02 | `src/components/documents/enhanced-status-badge.tsx` StatusCell | Playwright: `spec099-status-cell-fence`; keep `spec091-ingestion-surface` | LAW-099-3 |
| F-099-03 | `src/hooks/use-file-upload.ts` toast demotion | Playwright: `spec099-toast-demotion` | LAW-099-2/6 |
| F-099-04 | `document-dropzone.tsx` always-on `data-collapsed` band | Playwright: `spec099-upload-collapse` + `spec099-ux-audit-dropzone` | LAW-099-4 |
| F-099-05 | Clear All in header overflow | Playwright: `spec099-clear-all-demoted` | LAW-099-5 |
| F-099-06 | `ingestion-run-card` / `active-runs-panel` compact density | Playwright: `spec099-feedback-viewport` | LAW-099-2/4 |
| F-099-07 | `document-table-row` hide `spec048-row-stage` when live | Playwright: `spec099-live-row-no-stage-subtitle` | LAW-099-2 |
| F-099-08 | `use-documents-inventory` + `use-live-work-controllers` + actions context | shell composition; non-regression suite | LAW-099-9 |
| F-099-09 | `inventory-view-model.ts` overflow | unit + Playwright: `spec099-scale-overflow` | LAW-099-7 |
| F-099-10 | inventory view-model counts | Playwright: `spec099-filter-count-parity` | LAW-099-8 |
| F-099-11 | NEW badge removed | covered by table row change | LAW-099-4 |
| F-099-12 | Cost default-hidden + preference toggle | prefs + overflow menu | progressive disclosure |
| F-099-13 | `ux-ui-audit.spec.ts` → `document-dropzone` | Playwright: `spec099-ux-audit-dropzone` | LAW-099-10 |
| F-099-16 | Inventory flex chain: no Fragment; pinned chrome + internal table scroll | Playwright: `spec099-documents-scroll-layout` | LAW-099-4 |
| F-099-17 | Refresh CLS: reserve feedback slot + soft-refresh placeholderData | Playwright: `spec099-layout-stability` · unit: `documents-layout-stability` | LAW-099-4 |
| F-099-14 | `demotePipelineBanner` when zone open | Playwright: `spec099-banner-demote` | LAW-099-2 |
| F-099-15 | Failed highlight via domain display status | unit + keep `spec098-bulk-delete-honesty` | LAW-099-1 · LAW-098-11 |
| F-099-16 | Selection replaces primary toolbar | Playwright: `spec099-selection-toolbar` | LAW-099-9 |

## Non-regression anchors (must stay green)

| Prior gate | Protects |
|------------|----------|
| `e2e/spec048-ingestion-progress.spec.ts` | Active runs, quiet/`data-quiet`, phase parity |
| `e2e/spec050-delete-feedback-zone.spec.ts` | Delete in feedback zone |
| `e2e/spec086-ingestion-ux.spec.ts` | Dual-run, cancel, Needs attention |
| `e2e/spec091-ingestion-surface.spec.ts` | Serving fence Ready vs Indexed **truth** |
| `e2e/spec098-bulk-delete-honesty.spec.ts` | Mid-delete ≠ Completed/Ready |
| `e2e/spec350-bulk-upload-webui.spec.ts` | Multi-file drop → table |
| `src/lib/documents/__tests__/status-domain.test.ts` | Domain predicates |
| `src/lib/documents/__tests__/merge-monotonic-list.test.ts` | List merge |
| `src/lib/documents/__tests__/deletion-session.test.ts` | Pins/sessions |

## Issue ↔ finding

| Issue | Findings |
|-------|----------|
| [ISSUE-status-ssot-unify](issues/ISSUE-status-ssot-unify.md) | F-099-01, F-099-15 |
| [ISSUE-serving-fence-presentation](issues/ISSUE-serving-fence-presentation.md) | F-099-02 |
| [ISSUE-feedback-zone-density](issues/ISSUE-feedback-zone-density.md) | F-099-03, F-099-06, F-099-07, F-099-14 |
| [ISSUE-upload-slot-collapse](issues/ISSUE-upload-slot-collapse.md) | F-099-04, F-099-03 (toast) |
| [ISSUE-destructive-action-hierarchy](issues/ISSUE-destructive-action-hierarchy.md) | F-099-05, F-099-16 |
| [ISSUE-documents-shell-srp](issues/ISSUE-documents-shell-srp.md) | F-099-08 |
| [ISSUE-inventory-scale-honesty](issues/ISSUE-inventory-scale-honesty.md) | F-099-09, F-099-10, F-099-11, F-099-12, F-099-13 |
