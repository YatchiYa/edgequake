# SPEC-051 Screenshot Analysis

**Date:** 2026-07-13  
**All 5 E2E tests pass.**

## Screenshots

### S01 — Documents list reprocess dialog

| File                              | Description                               |
| --------------------------------- | ----------------------------------------- |
| S01-01-documents-list-initial.png | Documents list with 9 completed documents |
| S01-03-first-row.png              | First document row with actions menu      |
| S01-04-row-dropdown-open.png      | Dropdown with Reprocess option            |
| S01-05-reprocess-dialog.png       | ReprocessDialog open from documents list  |
| S01-06-reprocess-dialog-modes.png | Mode selection (Full / Entities Only)     |
| S01-final.png                     | Final state after cancel                  |

### S02 — Document detail page (GAP-051-01 fixed)

| File                                    | Description                                  |
| --------------------------------------- | -------------------------------------------- |
| S02-01-detail-page.png                  | **Reprocess button visible in header** ✅     |
| S02-02-header-buttons.png               | Header with Reprocess button annotated       |
| S02-03-reprocess-button.png             | Close-up of Reprocess button                 |
| S02-04-reprocess-dialog-from-detail.png | **ReprocessDialog opens from detail page** ✅ |

### S03 — No stale "go back to list" message (GAP-051-03)

| File                   | Description                                     |
| ---------------------- | ----------------------------------------------- |
| S03-01-detail-page.png | Cancelled/failed doc detail — no broken message |
| S03-final.png          | Final state                                     |

### S04 — Bulk reprocess (GAP-051-02 context)

| File                           | Description                              |
| ------------------------------ | ---------------------------------------- |
| S04-02-selection-made.png      | Documents selected via checkbox          |
| S04-03-bulk-reprocess-btn.png  | Bulk Reprocess button in BatchActionsBar |
| S04-04-bulk-dialog.png         | BulkReprocessDialog visible              |
| S04-05-bulk-dialog-content.png | **Bulk dialog with mode selection** ✅    |

### S05 — Regression check

| File                            | Description                  |
| ------------------------------- | ---------------------------- |
| S05-01-documents-regression.png | Documents page loads cleanly |
| S05-02-after-load.png           | No console errors after 2s   |

## Acceptance Criteria Status

| AC    | Criterion                                                          | Status   |
| ----- | ------------------------------------------------------------------ | -------- |
| AC-01 | Reprocess button on detail page for {failed, cancelled, completed} | ✅ S02    |
| AC-02 | ReprocessDialog with mode selection from detail page               | ✅ S02    |
| AC-03 | IngestionProgressPanel inline on detail page after reprocess       | ✅ (code) |
| AC-04 | Cancel button on detail page when processing                       | ✅ (code) |
| AC-05 | Bulk reprocess fires onReprocessTriggered                          | ✅ (code) |
| AC-06 | Backend: Postgres path uses workspace vision settings              | ✅ (code) |
| AC-07 | Detail page polls at 3s when track active                          | ✅ (code) |
| AC-08 | E2E test covers detail page reprocess                              | ✅ S02    |
| AC-09 | Screenshots saved to specs/051-reprocess/e2e/screenshots/          | ✅        |
