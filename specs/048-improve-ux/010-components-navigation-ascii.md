# 010 — Components & Navigation (ASCII)

---

## 1. Component map (target)

```text
DocumentManager
├── DocumentHeader
│   ├── WorkspaceBreadcrumb
│   ├── DocumentsTitle + count + liveDot
│   └── HeaderActions
│       ├── PipelineActivityPill     ← GET /pipeline/activity
│       ├── RefreshButton
│       └── ClearAllButton
│
├── DocumentToolbarSection
│   ├── DocumentFilters (search, status, sort)
│   ├── IngestionAlertBanner         ← IngestionRunView (primary active)
│   ├── DocumentDropzone             ← quiet when Working
│   └── ActiveRunsPanel              ← replaces UploadProgressList post-track
│       └── IngestionRunCard
│           ├── ServerStageStepper
│           ├── DeterminateBar (counts)
│           └── CancelRunButton
│
├── DocumentTableSection
│   └── DocumentTableRow
│       ├── TitleCell
│       ├── StatusCell               ← RunView overlay | Completed badge
│       ├── EntitiesCell
│       ├── CostCell
│       ├── CreatedCell / UpdatedCell
│       └── RowActionsMenu
│
└── Dialogs
    ├── PipelineStatusDialog         ← activity + task list
    └── IngestionRunDialog           ← timeline (screen 009-C)
```

---

## 2. Navigation / entry points

```text
  /documents
       │
       ├─► click banner [Open run] ──► IngestionRunDialog(track_id)
       ├─► click PipelineActivityPill ──► PipelineStatusDialog
       ├─► click row Status (if Working) ──► IngestionRunDialog
       ├─► row ⋮ → Reprocess ──► confirm mode → start run → banner updates
       └─► dropzone upload ──► client phase → track_id → ActiveRunsPanel
```

---

## 3. Data flow (props / stores)

```text
                    usePipelineActivity() ──┐
                    useDocumentsQuery() ────┤
                    useProgressWebSocket() ─┼──► buildIngestionRunViews()
                    usePdfProgress() ───────┘            │
                                                        ▼
                                              Map<documentId, RunView>
                                                        │
              ┌─────────────────┬───────────────────────┼──────────────┐
              ▼                 ▼                       ▼              ▼
         Banner            ActivityPill            RunCard         StatusCell
```

**Rule:** No consumer invents stage strings; all read `RunView.message` / `RunView.stage`.

---

## 4. Upload → run morph

```text
  t0  file selected
      UploadProgressList: Reading ●
  t1  bytes on wire
      Uploading ●
  t2  API returns track_id + document_id
      MORPH → ActiveRunsPanel + IngestionRunView(stage=queued|preprocessing)
      UploadProgressList unmounts for that file
  t3+ server stages drive ServerStageStepper
```

---

## 5. i18n keys (minimum)

```text
ingestion.stage.queued
ingestion.stage.uploading
ingestion.stage.converting
ingestion.stage.preprocessing
ingestion.stage.chunking
ingestion.stage.extracting
ingestion.stage.gleaning
ingestion.stage.merging
ingestion.stage.summarizing
ingestion.stage.embedding
ingestion.stage.storing
ingestion.stage.completed
ingestion.stage.failed
ingestion.busy.working
ingestion.busy.idle
ingestion.mode.full
ingestion.mode.entities
ingestion.mode.merge
ingestion.counts.chunks
ingestion.counts.pages
ingestion.banner.working
```

Cross-ref: [006 FE](./006-lens-frontend.md) · [009 screens](./009-screens-ascii.md)
