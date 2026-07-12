# 008 — Lens: Full Stack

**Job:** end-to-end progress contract — FE projection ≡ BE emission  
**Owns:** cross-cutting invariants PO-01…PO-05

---

## 1. System diagram (target)

```text
                    ┌─────────────────────────────────────┐
                    │           Documents UI              │
                    │  IngestionRunView (single model)    │
                    └──────────────┬──────────────────────┘
                                   │
           ┌───────────────────────┼───────────────────────┐
           │                       │                       │
           ▼                       ▼                       ▼
    WS ProgressEvent      GET /pipeline/activity    GET /ingestion/{id}/progress
    (ticks + stages)      (Busy SSOT)               (run detail / poll fallback)
           │                       │                       │
           └───────────────────────┼───────────────────────┘
                                   ▼
                    ┌─────────────────────────────────────┐
                    │  edgequake-api ProgressFacade       │
                    │  - KV document stage fields         │
                    │  - task manager                     │
                    │  - pipeline callbacks               │
                    └──────────────────┬──────────────────┘
                                       ▼
                    ┌─────────────────────────────────────┐
                    │  pipeline + pdf + merge (SPEC-047)  │
                    └─────────────────────────────────────┘
```

---

## 2. Cross-ref matrix (lenses → deliverables)

| Concern | PO | UX | UI | FE | BE | FS |
|---------|----|----|----|----|----|-----|
| Busy truth | PO-01 | anxiety | pill | alertMode | PipelineActivity | invariant test |
| Stage parity | PO-02 | IA | banner=row | RunView | UnifiedStage | e2e |
| Counts N/M | J1 | signal #3 | bar | counts | WS bridge | contract |
| Reprocess | PO-03 | honesty | mode badge | mode field | stage reset | e2e soft |
| Dead route | PO-04 | — | — | feature detect | implement/remove | network |
| Soft merge | PO-05 | microcopy | badge | mode | P7e DTO | bench UX |

---

## 3. Failure modes (fullstack)

| Failure | Symptom | Detection | Mitigation |
|---------|---------|-----------|------------|
| WS drop | Stale banner | heartbeat age >15s | Poll progress + activity |
| KV lag | Row Completed, banner Working | status≠stage | Prefer stage while processing |
| Partial WS | Extract silent | no ChunkProgress | Poll + PDF SSE |
| 404 progress | Console noise | network | Feature flag / implement |
| Double Busy | Pill on, table all Completed | PO-01 fail | Fix activity DTO |

---

## 4. Observability

Log/metric fields (structured):

```text
ingest.progress.emit  stage=… document_id=… track_id=… current=… total=…
ingest.ui.skew        banner_stage=… row_stage=… delta_ms=…
ingest.busy.false_positive  tasks=0 docs_processing=0
```

---

## 5. Definition of Done (fullstack)

- [ ] One `IngestionRunView` drives banner, row, active card  
- [ ] `PipelineActivity.busy` matches UI pill  
- [ ] Progress route exists **or** FE never calls it  
- [ ] Chunk + graph WS events observed in e2e  
- [ ] Soft-reprocess shows mode + reset stage  
- [ ] Acceptance matrix [013](./013-acceptance-criteria.md) green  

Cross-ref: [014 roadmap](./014-implementation-roadmap.md)
