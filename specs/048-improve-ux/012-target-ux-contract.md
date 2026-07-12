# 012 — Target UX Contract (normative)

**Audience:** FE + BE implementers  
**Status:** normative for SPEC-048 P0+

---

## 1. Single stage vocabulary

User-facing stages **MUST** match `UnifiedStage` in
`edgequake-pipeline/src/ingestion_types.rs` plus admission `queued`:

```text
queued | uploading | converting | preprocessing | chunking
| extracting | gleaning | merging | summarizing
| embedding | storing | completed | failed
```

Wire values: lowercase enum variants (e.g. `extracting`).  
Display labels: prefer `UnifiedStage::display_name()` / i18n `ingestion.stage.*`
(see [010](./010-components-navigation-ascii.md)).  
Free-text `stage_message` is **detail only**, never a substitute for `stage`.  
`converting` is PDF-only (skipped for MD/text — show muted skip, not error).

---

## 2. `IngestionProgress` DTO

```json
{
  "track_id": "uuid",
  "document_id": "uuid",
  "filename": "areal_2807.01120v2.pdf",
  "source_type": "pdf",
  "stage": "extracting",
  "stage_status": "active",
  "message": "Extracting Entities",
  "counts": { "current": 42, "total": 351, "unit": "chunks" },
  "progress_01": 0.12,
  "mode": "full",
  "cost_usd": 0.12,
  "updated_at": "2026-07-11T07:12:01Z"
}
```

| Field | Rule |
|-------|------|
| `stage` | Required; enum above |
| `stage_status` | `pending` \| `active` \| `complete` \| `failed` \| `skipped` |
| `counts` | Required when stage is countable and total known; else omit |
| `progress_01` | Only if determinate; else omit (UI shows indeterminate) |
| `mode` | Present on reprocess runs: `full` \| `entities` \| `merge` |
| `message` | Short; no raw enum dump |

---

## 3. `PipelineActivity` DTO

```json
{
  "busy": true,
  "working": [ { "document_id": "…", "filename": "…", "stage": "extracting" } ],
  "queued": [],
  "tasks": [ { "id": "…", "kind": "extract", "document_id": "…" } ],
  "updated_at": "…"
}
```

**Invariant:** `busy == (working.length + tasks.length > 0)`  
Queued-only ⇒ `busy=false` but UI may show Queued pill (not Busy).

---

## 4. Transport contract

| Channel | Must carry | Fallback |
|---------|------------|----------|
| WS `ProgressEvent` | stage change + countable ticks (chunk, page, merge) | poll progress |
| `GET /ingestion/{track_id}/progress` | full DTO | — (must exist if FE calls) |
| `GET …/pipeline/activity` | Busy SSOT | derive from documents (deprecated) |
| PDF SSE | page counts during preprocessing | WS/KV |
| Document list KV | `status`, `current_stage`, `stage_message`, `stage_progress` | — |

**Deprecation:** FE **MUST NOT** call progress URL if 404 feature-detect fails closed to KV+WS only.

---

## 5. UI projection rules

1. Banner primary run = first of `activity.working` else first client upload without track.  
2. Table StatusCell for that `document_id` **MUST** show same `stage` (+ counts if any).  
3. Header pill: Working if `busy`; else Queued if `queued.length>0`; else Idle.  
4. Upload 4-step legend **MUST** unmount after `track_id` (morph to server stepper).  
5. Never show `Completed` badge while `stage` ∉ {completed, failed} and `status=processing`.

---

## 6. Reprocess contract

On accept:

1. Persist reset fields (status/stage/message/progress).  
2. Emit WS event before long work.  
3. Include `mode` in all subsequent progress DTOs.  

---

## 7. Error / partial

```json
{
  "stage": "failed",
  "stage_status": "failed",
  "message": "12 chunks failed · Retry available",
  "counts": { "current": 339, "total": 351, "unit": "chunks" }
}
```

Partial success: `status=completed` with warning flag **OR** dedicated `partial` — pick one in P1; until then prefer explicit failed+retry.

Cross-ref: [008](./008-lens-fullstack.md) · [013](./013-acceptance-criteria-crossref.md)
