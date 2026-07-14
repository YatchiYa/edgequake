# 006 — Lens: Frontend

**Job:** one projection model; fewer transports lying to each other  
**Stack:** React 19 · React Query · Zustand · WS/SSE

---

## 1. Current component tree (law)

```text
documents/page.tsx
└── DocumentManager
    ├── DocumentHeader
    │   ├── ConnectionStatus
    │   ├── PipelineBusyButton ──► PipelineStatusDialog
    │   └── ConnectionBanner
    ├── DocumentToolbarSection
    │   ├── DocumentFilters
    │   ├── IngestionAlertBanner   ◄── resolvePipelineUiState
    │   ├── DocumentDropzone
    │   └── UploadProgressList
    │       ├── PdfUploadProgress  ◄── usePdfProgress
    │       └── IngestionProgressPanel ◄── useIngestionProgress (404 risk)
    └── DocumentTableSection
        └── DocumentTableRow
            ├── EnhancedStatusBadge ◄── useIngestionStore + doc
            └── CostCell
```

---

## 2. Target: `IngestionRunView` (FE SSOT)

```typescript
// Normative shape (SPEC-048) — implement in lib/pipeline/ingestion-run-view.ts
type IngestionRunView = {
  documentId: string;
  trackId: string | null;
  filename: string;
  sourceType: 'pdf' | 'markdown' | 'text' | 'image';
  stage: UnifiedStage | 'queued';  // uploading|converting|…|storing|completed|failed
  stageStatus: 'pending' | 'active' | 'complete' | 'failed' | 'skipped';
  message: string;           // single microcopy (display_name or detail)
  counts?: { current: number; total: number; unit: 'pages' | 'chunks' | 'entities' | 'relationships' };
  progress01?: number;       // only if determinate
  mode?: 'full' | 'entities' | 'merge';  // reprocess
  costUsd?: number;
  updatedAt: string;
};
```

**Projections:**

| Consumer | Uses |
|----------|------|
| Banner | Primary active run from `runs.filter(active)[0]` |
| Header pill | `alertMode` from runs + tasks |
| Table row | `runs.byDocumentId[id]` overlay on `Document` |
| Upload strip | Client phase until `trackId`, then same `IngestionRunView` |

---

## 3. Transport policy (FE)

```text
Priority for building IngestionRunView:
  1. WS ProgressEvent (chunk/merge/page) if connected
  2. PDF SSE / PdfUploadProgress for PDF tracks
  3. GET /documents fields (KV)
  4. Client upload FSM only pre-trackId

FORBIDDEN: inventing stages not in UnifiedStage.
FORBIDDEN: calling missing /ingestion/.../progress without feature detect.
```

---

## 4. Bugs to fix (FE-owned)

| DEF | Fix |
|-----|-----|
| DEF-05 | Render banner from `documents` alone if pipeline query pending |
| DEF-06 | Tab title: `workingCount` vs `queuedCount` |
| DEF-07 | Filter counts via `getDocumentDisplayStatus` |
| DEF-08 | Remove duplicate subline OR badge message |
| DEF-09 | Add missing `en.json` keys |
| DEF-10 | Morph upload stepper → server stepper |

---

## 5. Testing (FE)

- Unit: `resolvePipelineUiState` Busy⇒active invariant  
- Unit: `buildIngestionRunView` from fixture docs  
- Playwright: upload PDF → banner stage matches row within 5s  
- Network: no 404 `/ingestion/*/progress` after feature flag  

Cross-ref: [008 fullstack](./008-lens-fullstack.md) · [012 contract](./012-target-ux-contract.md)
