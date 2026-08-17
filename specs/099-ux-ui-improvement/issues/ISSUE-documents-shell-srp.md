# ISSUE — Documents shell SRP split

| Field | Value |
|-------|-------|
| ID | ISSUE-documents-shell-srp |
| Findings | F-099-08 |
| Laws | LAW-099-9 |
| Wave | W6 |
| Status | Open |

## Problem

`document-manager.tsx` (~1090 LOC) is a god-composer: upload, delete, reprocess, preview, filters, feedback zone, and table wiring. Toolbar recomputes `resolvePipelineUiState` independently. Table receives ~20 action callback props. Dead `activeRunViews` state remains.

## Why it hurts UX

Indirect: every honesty/disclosure fix is higher risk and slower; dual pipeline busy detection causes quiet/collapse and banner demote to disagree briefly.

## Approach

```ascii
DocumentsPageShell
├── inventory controller   (queries, filter VM, counts)
├── live-work controllers  (runs, upload, reprocess, delete)
├── DocumentsToolbar
├── DocumentsUploadSlot
├── DocumentsFeedbackZone
├── DocumentsInventoryTable + DocumentsActionsContext
└── DocumentPreviewDrawer
```

1. Extract controllers without behavior change first (strangler).  
2. Single `resolvePipelineUiState` shared by toolbar + shell.  
3. Replace prop drilling with actions context/store.  
4. Delete dead computed state.  
5. Keep all non-regression Playwright green before/after.

## DoD

- [ ] Manager (or successor shell) thin; zone/table/upload own SRP  
- [ ] One pipeline UI resolve  
- [ ] Row actions via context  
- [ ] 048/050/086/091/098/350 green  

## Non-goals

Rewriting ingestion store; changing WS protocol.
