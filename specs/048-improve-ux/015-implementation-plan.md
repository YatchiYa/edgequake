# 015 — Implementation Plan (DRY · SOLID · E2E screenshots)

**Status:** active  
**Implements:** [014 roadmap](./014-implementation-roadmap.md) P0 + P1 (P2 i18n keys included)  
**Out of scope here:** P3 timeline KV history

---

## 1. Architecture (SOLID)

```text
┌─────────────────────────────────────────────────────────────┐
│ Presentation (FE components)                                 │
│  Banner · HeaderPill · StatusCell · ActiveRunsPanel          │
│  — depend on IngestionRunView only (DIP)                     │
└──────────────────────────▲──────────────────────────────────┘
                           │ buildIngestionRunViews()
┌──────────────────────────┴──────────────────────────────────┐
│ Application projection (FE lib/pipeline/)                    │
│  ingestion-run-view.ts  — single builder (SRP)               │
│  pipeline-document-state.ts — alertMode / Busy (SRP)         │
└──────────────────────────▲──────────────────────────────────┘
                           │ HTTP / WS
┌──────────────────────────┴──────────────────────────────────┐
│ ProgressFacade (BE services/)                                │
│  assemble_track_progress · assemble_pipeline_activity (SRP)  │
│  — Open/Closed: new stages via UnifiedStage, not if-ladders  │
└──────────────────────────▲──────────────────────────────────┘
                           │
┌──────────────────────────┴──────────────────────────────────┐
│ Writers (existing)                                           │
│  status_updates · prepare · pipeline_progress_callback       │
│  + reprocess_stage_reset (new helper, DRY)                   │
│  + ws_bridge Chunk/Graph → ProgressEvent (ISP: thin adapter) │
└─────────────────────────────────────────────────────────────┘
```

| Principle | Application |
|-----------|-------------|
| **SRP** | Facade builds DTOs; handlers only HTTP; FE builder only projects |
| **OCP** | Stage labels from `UnifiedStage::display_name` / i18n map |
| **LSP** | `IngestionRunView` works for PDF + text + reprocess modes |
| **ISP** | Progress DTO ≠ full DocumentSummary; Activity ≠ EnhancedPipelineStatus |
| **DIP** | UI depends on RunView, not raw KV / WS shapes |
| **DRY** | One stage reset helper; one RunView builder; one Busy invariant |

---

## 2. Work packages

### WP-BE-1 — DTOs + ProgressFacade
- `handlers/ingestion_types.rs` — `IngestionProgressResponse`, counts, mode
- `handlers/pipeline_types.rs` — `PipelineActivityResponse`
- `services/progress_facade.rs` — assemble from KV + PipelineState + tasks

### WP-BE-2 — Routes
- `GET /api/v1/ingestion/{track_id}/progress`
- `GET /api/v1/pipeline/activity`
- Register in `routes.rs` + OpenAPI
- FE client: call `/api/v1/...` (or relative with existing baseURL)

### WP-BE-3 — Reprocess stage reset
- `processor` or `recovery` helper `reset_document_stage_for_reprocess(mode)`
- Patch: `status=processing`, `current_stage`, `stage_message`, `stage_progress=0`

### WP-BE-4 — WS bridge
- Add `ChunkProgress` + `GraphStorageProgress` to `ProgressEvent`
- Subscribe `PipelineState` → `ProgressBroadcaster` (mirror PdfPageProgress)

### WP-FE-1 — RunView SSOT
- `lib/pipeline/ingestion-run-view.ts` + unit tests
- Wire banner, header pill label, EnhancedStatusBadge overlay
- Fix DEF-05 (banner without pipelineStatus), DEF-06 (title counts)

### WP-FE-2 — Active run chrome (P1)
- `ServerStageStepper` + morph upload list when `trackId` set
- i18n `ingestion.stage.*` + missing `pipeline.*` keys

### WP-TEST — Contract + Playwright screenshots
- Rust: `contract_spec048_progress.rs`, `contract_spec048_activity.rs`, reprocess reset
- Playwright: `e2e/spec048-ingestion-progress.spec.ts`
- Artifacts: `specs/048-improve-ux/e2e/screenshots/*.png` + `ANALYSIS.md`

---

## 3. Screenshot matrix (analyzed)

| ID | Scenario | Capture | Assert |
|----|----------|---------|--------|
| S01 | Idle documents | full viewport | No Busy pill; no banner |
| S02 | Working banner + row parity | banner + active row | Same stage text |
| S03 | ActiveRunsPanel stepper | panel | Server stages visible; no 4-step client legend |
| S04 | Queued-only | header | Queued pill, not Busy |
| S05 | Stuck | banner | Stuck CTA visible |
| S06 | Pipeline activity dialog | dialog | Working list matches banner |

Analysis file per run: stage labels, Busy invariant, DEF regressions.

---

## 4. Test commands

```bash
# Backend contracts
cargo test -p edgequake-api --test contract_spec048_progress --test contract_spec048_activity

# FE unit
cd edgequake_webui && pnpm exec vitest run src/lib/pipeline/__tests__/ingestion-run-view.test.ts

# Playwright (writes screenshots into specs/048-improve-ux/e2e/screenshots)
cd edgequake_webui && SPEC048_SCREENSHOT_DIR=../specs/048-improve-ux/e2e/screenshots \
  pnpm exec playwright test e2e/spec048-ingestion-progress.spec.ts
```

---

## 5. Definition of Done

- [x] AC-01…AC-06, AC-08…AC-10 green (AC-07 mode badge best-effort)
- [x] Screenshots S01–S06 present + ANALYSIS.md reviewed
- [x] Contract `contract_spec048_progress` + FE unit `ingestion-run-view`
- [x] CHANGELOG + 000-index link to this plan
- [ ] Clippy clean on touched crates (run before merge)
- [ ] Live backend restart to load new routes in smoke workspace

Cross-ref: [012 contract](./012-target-ux-contract.md) · [013 AC](./013-acceptance-criteria-crossref.md) · [e2e ANALYSIS](./e2e/screenshots/ANALYSIS.md)
