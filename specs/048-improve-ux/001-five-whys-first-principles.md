# 001 — Five WHYs + First Principles

**Audience:** all lenses  
**Output:** axioms that every later decision must cite

---

## 1. Five WHYs (anchored in observed UI)

**Symptom (screenshot 2026-07-11):** Header shows **Pipeline Busy**; banner says *Processing 1 document — Extracting entities…*; table shows many **Completed** rows with entity counts; upload zone shows a separate **Reading → Uploading → Extracting → Done** stepper for `areal_2807.01120v2.pdf`.

### WHY 1 — Why does the user feel confused?

Because **three surfaces tell three stories** at once: global busy pill, workspace banner, per-file upload stepper, and per-row status — without a single shared “current work item” identity.

### WHY 2 — Why do the surfaces disagree?

Because they read **different sources of truth**:

| Surface | Source (code) |
|---------|----------------|
| Pipeline Busy | `resolvePipelineUiState` ← docs + `/tasks` / `/pipeline/status` |
| Banner detail | `stage_message` from polled `GET /documents` |
| Upload stepper | Client `UploadingFile` FSM (`use-file-upload`) + PDF 6-phase |
| Table badge | `current_stage` \|\| `status` + heuristics on `stage_message` |

### WHY 3 — Why are there multiple sources?

Because progress was **grown organically**:

1. KV metadata patches for the documents table (`status_updates.rs`)
2. `PdfUploadProgress` 6-phase model for PDF modal/SSE (`edgequake-tasks/progress.rs`)
3. `ProgressBroadcaster` WS for job-level events (`websocket_types.rs`)
4. Client-only upload phases before `track_id` exists

No single **Progress Contract** was enforced across layers.

### WHY 4 — Why wasn’t one contract enforced?

Because **backend emits richer internal events than WS exposes**:

- `PipelineEvent::ChunkProgress` / `GraphStorageProgress` exist in `edgequake-tasks`
- `ProgressEvent` on `/ws/pipeline/progress` **omits** them
- Frontend still polls `/ingestion/{trackId}/progress` — **route missing** → silent degradation to KV poll

### WHY 5 — Why does that matter for product?

Because ingest wall-clock is **minutes to hours** (SPEC-047: 117p PDF, soft-reprocess ~10–26 min). Without trustworthy progression, users:

- Re-click Upload / Reprocess (double-flight storms)
- Misread Completed rows as “system idle” while one doc extracts
- Lose trust in Cost / Entity counts mid-flight

**Root cause (First Principles):**  
**Progress is not a UI concern — it is a distributed state machine.** The UI failed because the **state machine was never published as one contract**; each layer invented a projection.

---

## 2. First Principles (axioms)

| ID | Axiom | Implication |
|----|-------|-------------|
| **FP-01** | One work item → one identity | Every surface keys on `(document_id, track_id)` |
| **FP-02** | One stage vocabulary | User-facing stages ⊆ `UnifiedStage` + `queued` only |
| **FP-03** | Determinate when countable | Prefer `N/M` (pages, chunks, entities) over fake % |
| **FP-04** | Indeterminate only when unknown | Spinner + verb; never invent % |
| **FP-05** | Terminal is sticky | Completed/failed cannot be overwritten by late patches |
| **FP-06** | Busy = active work OR honest queue | Never Busy with 0 active docs and 0 processing tasks |
| **FP-07** | Partial failure is first-class | Show succeeded/failed/skipped counts |
| **FP-08** | Leave and return | Progress survives navigation (server state, not only client stepper) |
| **FP-09** | Code is law | Specs cite symbols; no vibe-based stages |
| **FP-10** | Cost of opacity ∝ duration | Longer stages need richer microcopy (extract ≫ upload) |

---

## 3. Causal diagram

```text
                    ┌─────────────────────────┐
                    │  User anxiety / re-click │
                    └────────────▲────────────┘
                                 │
              ┌──────────────────┴──────────────────┐
              │  Conflicting progress narratives     │
              └──────────────────▲──────────────────┘
                                 │
     ┌───────────────┬───────────┴───────────┬───────────────┐
     │               │                       │               │
 Upload FSM     Banner/KV poll          Table badge      Pipeline Busy
 (client)       (stage_message)      (status+stage)     (tasks+is_busy)
     │               │                       │               │
     └───────┬───────┴───────────┬───────────┴───────┬───────┘
             │                   │                   │
             ▼                   ▼                   ▼
      PdfUploadProgress    KV metadata        ProgressBroadcaster
      REST/SSE             patches            (incomplete vs PipelineEvent)
             │                   │                   │
             └───────────────────┴───────────────────┘
                                 │
                                 ▼
                    NO SINGLE PROGRESS CONTRACT
```

---

## 4. Design law derived from WHYs

1. **Collapse narratives** → one `IngestionRunView` model projected to banner / row / dialog / upload strip.  
2. **Bridge WS** → publish `ChunkProgress` + `GraphStorageProgress` on `ProgressEvent` (or stop claiming live chunk UI).  
3. **Ship `/ingestion/{track_id}/progress`** or remove the FE call.  
4. **Normalize Busy** → `resolvePipelineUiState` remains SSOT; backend `/pipeline/status` must match.  
5. **Reset stages on reprocess** → clear `current_stage` / `stage_message` / `stage_progress` when queueing.

---

## 5. External research anchors (2026)

Industry patterns that **confirm** the axioms above (not replace code-as-law):

| Source | Takeaway → FP |
|--------|----------------|
| [LogRocket — async workflows / pipelines](https://blog.logrocket.com/ux-design/ui-patterns-for-async-workflows-background-jobs-and-data-pipelines/) (2026-02) | Expose queued→running→success/fail/partial; N/M counters; leave-and-return; pipeline timeline (Fivetran-like) → FP-01, FP-03, FP-07, FP-08 |
| [UX of Waiting](https://timgraf.com/ux-design/the-ux-of-waiting-how-loading-states-progress-indicators-and-perceived-performance-shape-user-trust/) (2026) | Never fake %; indeterminate + verb when unknown; long jobs must not trap in modal → FP-03, FP-04, FP-08 |
| [Nielsen visibility of system status](https://heurilens.com/blog/nielsens-heuristics/visibility-of-system-status-users-abandon-silent-apps) | >10s needs stage/ETA; cancel for long ops → FP-06, FP-10 |
| [Eleken progress indicators](https://www.eleken.co/blog-posts/progress-indicator-ux) | Match determinate vs indeterminate to real measurability → FP-03, FP-04 |

**EdgeQuake delta:** research assumes one job model; our code has **four** (client FSM, PDF 6-phase, UnifiedStage, legacy status). SPEC-048 collapses to UnifiedStage + queued.

Cross-ref: [002 inventory](./002-code-is-law-inventory.md) · [012 contract](./012-target-ux-contract.md)
