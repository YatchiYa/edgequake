# SPEC-050 — UI Designer Lens

## Component Inventory

### New Components

| Component             | File                                             | Purpose                            |
| --------------------- | ------------------------------------------------ | ---------------------------------- |
| `DeleteConfirmDialog` | `components/documents/delete-confirm-dialog.tsx` | Impact preview + confirm           |
| `DeletionImpactCard`  | `components/documents/deletion-impact-card.tsx`  | Shows entity/chunk/vector counts   |
| `DeleteProgressPanel` | `components/documents/delete-progress-panel.tsx` | Phase stepper during delete        |
| `DeletePhaseStep`     | `components/documents/delete-phase-step.tsx`     | Single phase row (reused in panel) |
| `DeletingRowOverlay`  | `components/documents/deleting-row-overlay.tsx`  | Dimmed row state while pending     |

### Modified Components

| Component              | Change                                                          |
| ---------------------- | --------------------------------------------------------------- |
| `DocumentActionsMenu`  | Route delete → `DeleteConfirmDialog` instead of direct mutation |
| `DocumentTableRow`     | Accept `isDeleting` prop → dim + spinner badge                  |
| `ClearDocumentsDialog` | Add per-document progress list                                  |
| `DocumentManager`      | Track `deletingDocumentIds: Set<string>` state                  |

## Visual States: Document Row

```
Normal:
  ┌──────────────────────────────────────────────────────────────┐
  │ ☐  📄 research.pdf          [Completed ✓]   $0.024   [...]  │
  └──────────────────────────────────────────────────────────────┘

Deleting (pending mutation):
  ┌──────────────────────────────────────────────────────────────┐
  │ ☐  📄 research.pdf          [Deleting ⟳]   $0.024   [...]  │  ← dimmed 50%
  └──────────────────────────────────────────────────────────────┘

Processing (ingestion / reprocess — already exists SPEC-048):
  ┌──────────────────────────────────────────────────────────────┐
  │ ☐  📄 research.pdf          [Processing ●]  $0.012  [...]   │
  │     □ uploading · □ converting · ● extracting · □ merging   │
  └──────────────────────────────────────────────────────────────┘

Queued (optimistic on reprocess confirm):
  ┌──────────────────────────────────────────────────────────────┐
  │ ☐  📄 research.pdf          [Queued ●]      —       [...]   │
  │     Waiting for a free worker slot…                         │
  └──────────────────────────────────────────────────────────────┘

Partial failure (delete finished but graph had error):
  ┌──────────────────────────────────────────────────────────────┐
  │ ☐  📄 research.pdf          [Partial ⚠]    $0.024  [...]   │
  └──────────────────────────────────────────────────────────────┘
```

## DeleteConfirmDialog ASCII Mockup

```
┌─────────────────────────────────────────────────────────────────────┐
│  Delete "research.pdf"?                                     [✕]     │
│─────────────────────────────────────────────────────────────────────│
│  This action is permanent and cannot be undone.                     │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Impact Analysis                                             │   │
│  │─────────────────────────────────────────────────────────────│   │
│  │  📄  1 document          🔗  14 chunks                       │   │
│  │  🧩  34 vectors          🏷   8 entities (removed)           │   │
│  │  ↔   12 relationships (removed)                             │   │
│  │  🔄   3 entities updated (other sources remain)             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│                         [Cancel]  [Delete permanently]              │
└─────────────────────────────────────────────────────────────────────┘
```

Loading state (impact fetch in progress):
```
┌─────────────────────────────────────────────────────────────────────┐
│  Delete "research.pdf"?                                     [✕]     │
│─────────────────────────────────────────────────────────────────────│
│  This action is permanent and cannot be undone.                     │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  ⟳  Analyzing impact…                                       │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│                         [Cancel]  [Delete permanently ⟳]           │
└─────────────────────────────────────────────────────────────────────┘
```

## DeleteProgressPanel ASCII Mockup

```
┌─────────────────────────────────────────────────────────────────────┐
│  Deleting "research.pdf"                                            │
│─────────────────────────────────────────────────────────────────────│
│  ✓  Task cancelled                         done  0ms               │
│  ⟳  Removing 34 vector embeddings          active · 14/34          │
│  ·  Removing 8 graph entities                                       │
│  ·  Removing KV records                                             │
│  ·  Finalizing                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## ClearDocumentsDialog Progress ASCII Mockup

```
┌─────────────────────────────────────────────────────────────────────┐
│  Deleting all documents…                                            │
│─────────────────────────────────────────────────────────────────────│
│  ████████████░░░░░░░░░░░░  7 / 23 documents                         │
│                                                                     │
│  ✓ research.pdf                      8 entities, 12 rels removed    │
│  ✓ manual.docx                       2 entities, 1 rel removed      │
│  ⟳ specification.md            (removing graph…)                    │
│  · contract.pdf               (pending)                             │
│  · notes.txt                  (pending)                             │
│                                                                     │
│  ⚠ 2 documents skipped (currently processing)                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Color Tokens (existing — reuse)

| State         | Badge class                     | Dot class                     |
| ------------- | ------------------------------- | ----------------------------- |
| deleting      | `bg-orange-100 text-orange-800` | `bg-orange-500 animate-pulse` |
| queued        | `bg-amber-100 text-amber-800`   | `bg-amber-500 animate-pulse`  |
| phase done    | `text-emerald-700`              | `bg-emerald-500`              |
| phase active  | `bg-sky-100 text-sky-800`       | `bg-sky-500 animate-pulse`    |
| phase failed  | `bg-rose-100 text-rose-800`     | `bg-rose-500`                 |
| phase pending | `text-muted-foreground`         | `bg-muted-foreground/40`      |

## Reprocess: Optimistic Queued State

When the user confirms reprocess, before any WS event arrives:

```typescript
// In useDocumentMutations — onMutate:
queryClient.setQueryData(['documents', ...], (old) => {
  // Optimistically mark the document as queued
  return { ...old, items: old.items.map(d =>
    d.id === variables.id
      ? { ...d, status: 'pending', current_stage: 'queued', track_id: <new uuid> }
      : d
  )};
});
```

This ensures the row immediately shows the "Queued" SPEC-048 stepper state.
