# 011 — State Machines (ASCII)

---

## 1. Document coarse status (existing)

```text
                 ┌──────────┐
                 │ pending  │
                 └────┬─────┘
                      │ accept upload / reprocess
                      ▼
                 ┌────────────┐
            ┌───►│ processing │◄──┐
            │    └─────┬──────┘   │
            │          │          │ retry / reprocess
            │    ┌─────┴──────┐   │
            │    ▼            ▼   │
            │ ┌──────┐   ┌─────────┐
            │ │failed│   │completed│
            │ └──┬───┘   └─────────┘
            │    │ cancel
            │    ▼
            │ ┌───────────┐
            └─│ cancelled │
              └───────────┘
```

---

## 2. UnifiedStage (fine) — happy path

Law: `UnifiedStage` in `ingestion_types.rs` (+ admission `queued`).

```text
  queued → uploading → converting* → preprocessing → chunking
       → extracting → gleaning → merging → summarizing
       → embedding → storing → completed

  Any stage ──error──► failed
  * converting: PDF only; MD/text → skipped (not failed)
  Countable: pages (convert/preprocess), chunks (extract), merge units
  mode=merge (P7e): reuse snapshot → merging/storing path
```

---

## 3. UI alertMode (target)

```text
                    ┌─────────┐
                    │  idle   │  pill: Idle / hidden
                    └────┬────┘
                         │ activity.busy
                         ▼
                    ┌─────────┐
              ┌────►│ working │  pill: Working · N
              │     └────┬────┘
              │          │ all working done; queued remain
              │          ▼
              │     ┌─────────┐
              │     │ queued  │  pill: Queued · N
              │     └────┬────┘
              │          │ task starts
              └──────────┘
                         │
              stuck heuristic (no tick > T)
                         ▼
                    ┌─────────┐
                    │  stuck  │  pill: Stuck · CTA
                    └─────────┘
```

**Invariant:** `working` ⇒ ∃ document with processing-class stage OR ∃ processing task.

---

## 4. Client upload FSM → server (morph)

```text
  ┌─────────┐   file ok    ┌───────────┐   POST ok   ┌──────────────┐
  │ reading │─────────────►│ uploading │────────────►│ has track_id │
  └─────────┘              └───────────┘             └──────┬───────┘
       │ fail                    │ fail                     │
       ▼                         ▼                          ▼
  ┌─────────┐              ┌─────────┐              ┌─────────────────┐
  │  error  │              │  error  │              │ follow Unified  │
  └─────────┘              └─────────┘              │ Stage machine   │
                                                    └─────────────────┘
```

---

## 5. Reprocess start (BE must emit)

```text
  [user: Reprocess mode=M]
           │
           ▼
  status=processing
  current_stage = f(M)     // full→queued/extracting; merge→merging/storing
  stage_message = start(M)
  stage_progress = 0
  WS ProgressEvent
           │
           ▼
  pipeline run(M)
```

---

## 6. Progress tick (countable stages)

```text
  stage active
       │
       ▼
  ┌──────────────────────────────┐
  │ on unit complete             │
  │   counts.current += 1        │
  │   progress_01 = cur/total    │
  │   emit WS + patch KV         │
  └──────────────────────────────┘
       │
       ▼
  current == total → advance stage
```

Cross-ref: [007 BE](./007-lens-backend.md) · [012 contract](./012-target-ux-contract.md)
