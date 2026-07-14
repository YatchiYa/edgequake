# 002 — Code-is-law inventory

Every claim below cites a symbol. Paths are repo-relative from workspace root.

---

## 1. Stage vocabularies (collision map)

```text
┌────────────────────┬──────────────────────────────┬─────────────────────────┐
│ Vocabulary         │ SSOT symbol                  │ Path                    │
├────────────────────┼──────────────────────────────┼─────────────────────────┤
│ UnifiedStage       │ UnifiedStage                 │ edgequake-pipeline/     │
│ (frontend-facing)  │                              │ src/ingestion_types.rs  │
├────────────────────┼──────────────────────────────┼─────────────────────────┤
│ PipelineStage      │ PipelineStage                │ edgequake-pipeline/     │
│ (internal)         │                              │ src/progress/mod.rs     │
├────────────────────┼──────────────────────────────┼─────────────────────────┤
│ Legacy status      │ is_active_processing_status  │ edgequake-api/src/      │
│ string             │                              │ document_metadata.rs    │
├────────────────────┼──────────────────────────────┼─────────────────────────┤
│ Pdf 6-phase        │ PdfUploadProgress            │ edgequake-tasks/src/    │
│                    │                              │ progress.rs             │
├────────────────────┼──────────────────────────────┼─────────────────────────┤
│ Upload client FSM  │ UploadingFile.status         │ edgequake_webui/.../    │
│                    │ pending|reading|uploading|…  │ documents/types.ts      │
├────────────────────┼──────────────────────────────┼─────────────────────────┤
│ TaskStatus         │ TaskStatus                   │ edgequake-tasks/src/    │
│                    │                              │ types/status.rs         │
└────────────────────┴──────────────────────────────┴─────────────────────────┘
```

**Law:** User-facing copy MUST map to `UnifiedStage` (+ admission `queued`). Legacy `status` is a **storage compatibility field**, not a UX label.

### UnifiedStage flow (from code comment)

```text
[uploading] → [converting?] → [preprocessing] → [chunking]
      → [extracting] → [gleaning] → [merging] → [summarizing]
      → [embedding] → [storing] → [completed | failed]
```

Source: `edgequake-pipeline/src/ingestion_types.rs` L19–34.

### Legacy → unified mapping (writer)

`DocumentTaskProcessor::update_document_status` — `edgequake-api/src/processor/status_updates.rs`  
Examples: `indexing` → `current_stage: "storing"`; `processing` → `preprocessing`.

---

## 2. Progress writers (backend)

| Phase          | Writer                                | Fields touched                                  |
| ----------------| ---------------------------------------| -------------------------------------------------|
| Admission      | `ingest_admission.rs`                 | `status/current_stage=queued`                   |
| Upload shell   | `document_admission.rs`               | `uploading`, progress 0                         |
| PDF pages      | `pipeline_progress_callback.rs`       | `stage_message`, `stage_progress` (2s debounce) |
| Chunking       | `text_insert/prepare.rs`              | `chunking`                                      |
| Extract chunks | `prepare.rs` patch every 3 chunks     | `extracting`, `chunk N/M (P%)`                  |
| Embed          | `text_insert/extraction.rs`           | `embedding`, sub-stage counts                   |
| Graph merge    | `patch_document_graph_merge_progress` | `indexing` + `storing`, merge counters          |
| Complete       | `update_document_status_with_stats`   | terminal + stats                                |
| Reprocess      | `reprocess.rs`                        | **only** `pending` + new `track_id` ⚠           |

---

## 3. Progress readers (frontend)

| Surface | Component / hook | Transport |
|---------|------------------|-----------|
| Header Busy | `document-header.tsx` | `resolvePipelineUiState` |
| Banner | `ingestion-alert-banner.tsx` | docs poll + pipeline stats |
| Upload stepper | `upload-progress-list.tsx` | client FSM |
| PDF phases | `pdf-upload-progress.tsx` / `use-pdf-progress.ts` | WS + SSE + poll |
| Text panel | `ingestion-progress-panel.tsx` | `getTrackProgress` ⚠ 404 |
| Table badge | `enhanced-status-badge.tsx` | docs + `useIngestionStore` |
| Dialog | `pipeline-status-dialog.tsx` | `/pipeline/status` |

**Pipeline Busy SSOT (FE):** `edgequake_webui/src/lib/pipeline/pipeline-document-state.ts`  
`resolvePipelineUiState` — deliberately ignores stale `is_busy` alone (L171–175 region).

---

## 4. Transport matrix (truth table)

```text
                    ┌──────────┬──────────┬──────────┬──────────┐
                    │ KV poll  │ PDF SSE  │ WS Prog. │ /ingest… │
                    │ /docs    │ stream   │ Broadcst │ progress │
├───────────────────┼──────────┼──────────┼──────────┼──────────┤
 stage_message      │   YES    │   via    │  partial │  MISSING │
                    │          │  PdfProg │          │  route   │
 chunk N/M live     │  every 3 │   NO     │   NO*    │  MISSING │
 merge counters     │   YES    │   NO     │   NO*    │  MISSING │
 PDF page N/M       │   YES    │   YES    │   YES    │    —     │
 is_busy            │    —     │    —     │ snapshot │    —     │
└───────────────────┴──────────┴──────────┴──────────┴──────────┘
* ChunkProgress / GraphStorageProgress exist as PipelineEvent but are
  NOT variants of ProgressEvent (websocket_types.rs).
```

FE call: `getTrackProgress` → `GET /ingestion/{trackId}/progress`  
(`edgequake_webui/src/lib/api/edgequake/pipeline.ts`) — **no matching route in API**.

---

## 5. Document list DTO (progress fields)

`DocumentSummary` — `edgequake-api/src/handlers/documents_types/listing.rs`

| Field | Meaning |
|-------|---------|
| `status` | Legacy string |
| `current_stage` | Unified stage name |
| `stage_progress` | **0.0–1.0 fraction** (not percent) |
| `stage_message` | Free-text microcopy |
| `entity_count`, `cost_usd` | Outcome metrics |
| `track_id` | Live subscription key |

⚠ Relational backfill path may null SPEC-002 stage fields (`document_read_model.rs`) — list can lose progress if KV drifts.

---

## 6. Known defects (code-anchored)

| ID | Defect | Anchor |
|----|--------|--------|
| DEF-01 | `/ingestion/{id}/progress` 404 | FE `pipeline.ts`; absent in API routes |
| DEF-02 | Chunk/merge not on WS `ProgressEvent` | `websocket_types.rs` vs `pipeline_state/event.rs` |
| DEF-03 | Reprocess leaves stale `stage_message` | `reprocess.rs` KV patch |
| DEF-04 | Dual Busy semantics | BE `/pipeline/status` ORs snapshot; FE basic path uses `processing>0` only |
| DEF-05 | Banner gated on `pipelineStatus` truthy | `document-toolbar-section.tsx` — may hide while docs process |
| DEF-06 | Tab title counts queued as processing | `document-manager.tsx` + `isProcessingStatus` |
| DEF-07 | Filter counts use legacy `status===processing` | `use-document-filtering.ts` |
| DEF-08 | Duplicate `stage_message` under badge | `document-table-row.tsx` + badge tooltip |
| DEF-09 | Missing i18n keys (fallback English) | `pipeline.processing`, upload phases, etc. |
| DEF-10 | Upload stepper ≠ server stages | 4 client phases vs 10+ unified stages |

Cross-ref: [001 WHYs](./001-five-whys-first-principles.md) · [012 contract](./012-target-ux-contract.md)
