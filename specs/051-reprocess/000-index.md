# SPEC-051 — Reprocess: E2E UX/Backend Gap Closure

**Status:** ✅ COMPLETE  
**Branch:** feat/spec047-vision-ingest-spec048-progress  
**Date:** 2026-07-13  

## Problem Statement

Reprocess does **not** follow the same e2e pipeline feedback as a fresh upload.  
Multiple access points exist with inconsistent behaviour: detail page has no action at all,  
bulk reprocess misses progress panels, backend uses two different provider-resolution paths.

## Documents

| File                                                             | Content                       |
| ---------------------------------------------------------------- | ----------------------------- |
| [001-code-is-law-analysis.md](./001-code-is-law-analysis.md)     | First-principles gap analysis |
| [002-implementation-plan.md](./002-implementation-plan.md)       | Concrete tasks + DoD          |
| [e2e/spec051-reprocess.spec.ts](./e2e/spec051-reprocess.spec.ts) | Playwright E2E test           |
| [e2e/screenshots/](./e2e/screenshots/)                           | Captured screenshots          |

## Quick Summary of Gaps

| ID         | Gap                                                           | Severity | Fixed |
| ---------- | ------------------------------------------------------------- | -------- | ----- |
| GAP-051-01 | No reprocess action on document detail page                   | CRITICAL | ✅     |
| GAP-051-02 | Bulk reprocess bypasses IngestionProgressPanel                | HIGH     | ✅     |
| GAP-051-03 | No live progress panel on detail page during reprocess        | HIGH     | ✅     |
| GAP-051-04 | Backend: DRY violation in vision provider resolution          | MEDIUM   | ✅     |
| GAP-051-05 | `ResetDocumentStatusButton` hardcodes mode='full' (no dialog) | MEDIUM   | ✅     |
