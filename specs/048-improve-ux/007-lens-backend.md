# 007 — Lens: Backend

**Job:** emit one progress truth the UI can trust  
**Crates:** `edgequake-api` · `edgequake-pipeline` · `edgequake-tasks`

---

## 1. Dual vocabulary (law today)

```text
  Document.status (coarse)          UnifiedStage (fine) — ingestion_types.rs
  ─────────────────────────         ──────────────────────────────────────
  pending                           queued (admission) / uploading
  processing                        converting (PDF) → preprocessing → chunking
  completed                         extracting → gleaning → merging
  failed                            summarizing → embedding → storing
  cancelled                         completed | failed
```

**Problem:** UI mixes both. Banner often shows free-text `stage_message` while badge uses coarse `status`.

---

## 2. Writers (code anchors)

| Writer | Path | Emits |
|--------|------|-------|
| `status_updates.rs` | KV document fields | `status`, `current_stage`, `stage_message`, `stage_progress` |
| Pipeline stages | `text_insert/*`, `pdf_processing` | stage transitions |
| Task manager | `edgequake-tasks` | processing tasks for Busy |
| WS hub | `ProgressEvent` | **partial** — gaps on Chunk/Graph |
| PDF SSE | pdf track | page N/M |

---

## 3. Defects (BE-owned)

| DEF | Root | Fix |
|-----|------|-----|
| DEF-01 | `GET /ingestion/{trackId}/progress` missing | Add route **or** deprecate FE call |
| DEF-02 | WS missing ChunkProgress / GraphStorageProgress | Bridge from pipeline callbacks |
| DEF-03 | Reprocess doesn’t reset stage fields | Clear `current_stage`/`stage_message`/`stage_progress` on reprocess start |
| DEF-04 | Busy = tasks OR docs; docs lag | Single `PipelineActivity` DTO from server |

---

## 4. Target: `PipelineActivity` + `IngestionProgress` APIs

```text
GET /workspaces/{ws}/pipeline/activity
→ {
    busy: bool,
    working_documents: [{ id, filename, stage, message, counts? }],
    queued_documents: [...],
    processing_tasks: [{ id, kind, document_id? }],
    updated_at
  }

GET /ingestion/{track_id}/progress   (implement)
→ {
    track_id, document_id, stage, stage_status,
    message, counts?, progress_01?, mode?, cost_usd?,
    updated_at
  }
```

**WS:** every stage transition + countable tick emits `ProgressEvent` with same shape as progress DTO (subset).

---

## 5. Stage reset contract (reprocess)

```text
On reprocess accept (any mode):
  1. status = processing
  2. current_stage = queued | extracting | merging | storing  (by mode)
  3. stage_message = mode-specific start string
  4. stage_progress = 0
  5. emit WS ProgressEvent
  6. then run pipeline
```

Modes (SPEC-047 P7e): `full` · `entities` · `merge` — surface in progress DTO.

---

## 6. Testing (BE)

- Contract: progress route 200 for live track  
- Contract: reprocess resets stage fields before first extract tick  
- E2E: WS receives ChunkProgress during extract  
- Unit: Busy iff working_docs ∪ tasks non-empty  

Cross-ref: [002 inventory](./002-code-is-law-inventory.md) · [012 contract](./012-target-ux-contract.md)
