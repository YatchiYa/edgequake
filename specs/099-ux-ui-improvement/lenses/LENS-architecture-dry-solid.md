# LENS — Architecture DRY / SOLID (SPEC-099)

## Question

Where does Documents UI violate DRY/SOLID in ways that create UX honesty bugs?

## Critical smell — dual status SSOT

```ascii
 status-domain.ts          status-badge.tsx
 ┌──────────────────┐      ┌──────────────────┐
 │ normalizeStatus  │      │ normalizeStatus  │  ← DUPLICATE
 │ isProcessing…    │      │ isProcessing…    │
 │ isTerminal…      │      │ isTerminal…      │
 │ getDocumentDisp… │      │ getDocumentDisp… │
 └────────┬─────────┘      └────────┬─────────┘
          │                         │
   merge-monotonic-list      document-status.ts
                             ingestion-run-view
                             hooks / tests
```

**Fix (LAW-099-1):** domain only; badge = `status → {icon,color,label}` map.

## God-composer — DocumentManager

| Smell | Evidence |
|-------|----------|
| SRP | ~1090 LOC wires upload, delete, reprocess, preview, filters, zone, table |
| Prop drilling | ~20 action props through table → row |
| DIP leak | Toolbar recomputes `resolvePipelineUiState` separately from manager |
| Dead state | Unused `activeRunViews` residue post auto-seed removal |

## Target boundaries

```ascii
DocumentsPageShell
├── useDocumentsInventory()      → rows, counts, filter VM
├── useLiveWorkControllers()     → runs, upload, reprocess, delete sessions
├── DocumentsToolbar
├── DocumentsUploadSlot
├── DocumentsFeedbackZone
├── DocumentsInventoryTable
│     └── DocumentsActionsContext  (row actions)
└── DocumentPreviewDrawer
```

## SOLID checklist (W6 DoD)

| Principle | Check |
|-----------|-------|
| DRY | One domain; one pipeline UI resolve; one inventory count VM |
| SRP | Shell does not own toast, stepper, or SQL merge |
| OCP | New status → domain + presentation config only |
| DIP | Table depends on `inventoryViewModel` + `isLiveRun`, not upload hook |
| ISP | Row context exposes only needed actions |
| LSP | Domain predicates identical for WS patch vs poll merge |

## Laws

LAW-099-1 · LAW-099-9 · LAW-099-10

## Cross-ref

F-099-01 · F-099-08 · Issues `ISSUE-status-ssot-unify`, `ISSUE-documents-shell-srp`
