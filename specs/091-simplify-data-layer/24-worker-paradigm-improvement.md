# 24 — Worker Paradigm: First-Principles Assessment + Improvement Plan

> **Status:** ASSESSMENT + **WP0–WP1 IMPLEMENTED** (2026-07-31). WP2–WP5 open. Code is law for WP0–WP1.
> **Scope:** Is the **worker-pool paradigm** the right way to implement EdgeQuake ingestion? First-principles analysis against Rust async practice (Tokio, July 2026), systems engineering (queues, leases, backpressure, durable execution), and SPEC-091 laws. Produces a concrete improvement program **WP0–WP5**.
> **Inherits:** [02](02-first-principles.md) · [12](12-queue-admission-first-principles.md)–[14](14-queue-admission-plan.md) · [20](20-ingestion-surface-assessment.md) · [21](21-ingestion-pipeline-data-model-improvement.md) · [22](22-ingestion-migration-system-assessment.md) · [23](23-post-drop-kv-hot-path.md).
> **Does not reopen:** Typed SSOT cutovers (A–D / IW*), fence default, outbox/RM waves, UI chrome (IS*). This doc owns **how work is scheduled and bounded**, not what rows are written.
> **Output:** verdict · axioms **A-WP1..5** · laws **LAW-WP1..WP8** · findings `F-WP-01..18` · waves **WP0–WP5** · acceptance `WP-AC-01..14` · locked decision **LD-18**.

### WP0–WP1 landing notes (2026-07-31)

| ID | Status | Evidence |
| --- | --- | --- |
| WP-AC-01 | Met | LD-18 + this doc indexed in README |
| WP-AC-02 | Met | `record_provider_slots_inflight` + `record_provider_slot_hold_duration` on `ProviderSlotGuard`; stage wall via `record_ingest_stage_duration(prepare\|extract\|persist\|finalize)` |
| WP-AC-03 | Met | Ledger path exclusive when installed (`gate_ledger_never_returns_semaphore_variant`); PG `contract_spec091_provider_budget` in `spec091-gates` |
| WP-AC-04 | Met | `processor/cancel_gates.rs` SSOT; `pre-prepare` / `pre-embed` wired; `contract_spec091_cancel_gates` |
| WP-AC-05 | Met | Relational + successful typed write → KV write-stop; `contract_spec091_checkpoint_typed_write_stop` |
| Fairness mid-flight | Met | `process_with_fairness` + drop before materialize; `contract_spec091_fairness_release_before_materialize` |

---

## 1. Verdict (one paragraph)

**The worker paradigm is the right *control plane*** for EdgeQuake ingestion: a durable Postgres job table, `FOR UPDATE SKIP LOCKED` claim, lease+heartbeat, wake channel, and a bounded set of Tokio tasks that execute work off the HTTP request lifetime. That shape matches production RAG practice, PostgreSQL queue guidance, and laws already locked in SPEC-091 (LD-08, LD-11..13, LAW-Q*). **What is wrong is the *execution shape* of today’s worker:** one claimed `Task` holds one worker for the *entire* document pipeline (PDF convert → chunk → LLM extract → embed → AGE merge → persist → fence), so heterogeneous bottlenecks share one concurrency knob, scarce provider capacity is coupled to wall-clock stage time that is not inference, crash resume is coarse (one post-extract checkpoint, still KV-keyed), and process-local fairness/admission machinery still shadows the cluster ledger. The correct target is **not** “replace workers with actors / Temporal / Kafka,” but **keep claim-workers and evolve them into stage-bounded, resource-aware execution** under one Job → StageAttempt state machine — durable orchestration *contracts* without adopting an external workflow engine unless measurement forces it.

```ascii
 TODAY (correct control plane, wrong unit of work)
 ┌──────── HTTP ────────┐     ┌──────── WorkerPool (N tasks) ────────┐
 │ admit + enqueue Task │────▶│ claim_next (SKIP LOCKED) + lease     │
 │ 202 + track_id       │     │ process() ══════════════════════════ │
 └──────────────────────┘     │   PDF | extract | embed | persist    │
                              │   ← one worker owns ALL stages →     │
                              └──────────────────────────────────────┘

 TARGET (same control plane, stage-bounded execution)
 ┌──────── HTTP ────────┐     ┌──── Claim workers (by stage class) ──┐
 │ admit Job + Stage 0  │────▶│ parse pool │ extract pool │ embed…   │
 │ 202 + job_id         │     │ each: claim → lease scarce resource  │
 └──────────────────────┘     │        → idempotent stage → next     │
                              │ provider_slot held ONLY in LLM/embed │
                              └──────────────────────────────────────┘
```

---

## 2. Method

| Lens | Question | Anchor (July 2026) |
| --- | --- | --- |
| L1 First principles | What must any ingestion system *minimally* guarantee? | Axioms below; [02](02-first-principles.md); [12](12-queue-admission-first-principles.md) |
| L2 Code-is-law | What does HEAD actually do on claim → process → complete? | `edgequake-tasks/src/worker.rs` (~2.2k LOC); `processor/task_impl.rs`; `text_insert/*`; `ingestion_persister.rs` |
| L3 Systems engineering | Queue vs pipeline vs workflow engine: which contracts matter? | Postgres `SKIP LOCKED` queues; ToolLeap RAG ingestion control plane; Rapidflare/Temporal stage isolation + sliding window |
| L4 Rust / Tokio | Is the runtime topology sound? | Tokio cooperative scheduler; classify I/O vs CPU; bounded channels; separate runtimes for serve vs ingest; `spawn_blocking` for CPU |
| L5 Capacity math | Is the scarce resource gated at the right grain? | Little’s Law; LAW-Q1/Q3; provider_slot ledger (mig 110) |
| L6 SOLID / DRY / SSOT | One transition authority, one budget, one unit of work identity | LAW-Q2; `state_machine.rs`; SPEC-120 `job.rs` types (unwired) |

Grades: **Strong** / **Partial** / **Weak** / **Wrong**.

---

## 3. First principles — what ingestion *must* be

Reduce the problem to axioms (independent of Tokio, Temporal, or “worker” as a brand):

| ID | Axiom | Consequence |
| --- | --- | --- |
| **A-WP1** | **Intent outlives the process.** An upload/cancel/delete must survive crash, deploy, and client disconnect. | Durable row + claim, not in-request futures. |
| **A-WP2** | **The scarce resource is not “a worker.”** It is provider inference, parse CPU/RAM, embed QPS, and DB write bandwidth — *different* ceilings. | Concurrency must be **per scarce resource**, not one global N. |
| **A-WP3** | **Partial success is normal.** Stages fail independently; retrying the whole document without stage identity wastes cost and risks duplicate side effects. | Stage-level idempotency + fence (LAW-D1/D3). |
| **A-WP4** | **Delivery is at-least-once; effects must be effect-once.** Leases expire; claims are redelivered. | Upserts, CAS transitions, fencing tokens — never “exactly-once queue” faith. |
| **A-WP5** | **Query latency must not share fate with ingest saturation.** | Separate runtime / pool budgets for serve vs ingest (already partially honored via `WorkerPool::start_on`). |

These axioms are exactly why production RAG literature puts **workers before GPUs**: the control plane (durability, idempotency, backpressure, visibility, replay) is the product; hardware only accelerates one stage.

---

## 4. Code-is-law: today’s worker topology

### 4.1 Control plane (grade: **Strong**)

| Mechanism | Location | Grade |
| --- | --- | --- |
| Durable task row + status machine | `state_machine.rs` (`TaskEvent`); SQL claim guards | **Strong** (LAW-Q2 mostly wired) |
| `claim_next` + `SKIP LOCKED` + lease token | `postgres.rs` / `worker.rs` claim loop | **Strong** |
| Wake channel is wake-only; claim authorizes | `worker.rs` select: receive → ignore payload → claim | **Strong** (SPEC-057 P1) |
| Lease heartbeat + lost-lease abort | `HeartbeatGuard` + `refresh_lease` | **Strong** |
| Provider budget ledger | `provider_budget.rs` + mig 110 | **Strong** design (LAW-Q3) |
| Fairness park (release claim, wait, handoff) | `FairnessParkSet` / `PermitHandoff` | **Partial** — correct idea, **process-local** park set |
| Byte admission | `InFlightByteBudget` | **Partial** — process-local |
| Cancel intent at claim / park / stage | cancel registry + processor checks | **Partial** — convention, not enumerated gate suite |
| Multi-replica exactly-once process | `contract_multi_replica_claim` | **Strong** for *document* claim |

### 4.2 Execution unit (grade: **Weak**)

```ascii
 DocumentTaskProcessor::process(task)
   ├─ PdfProcessing → convert pages (CPU + vision LLM) ── long pole
   ├─ TextInsert prepare → extract (LLM, minutes) ─────── holds worker
   │     checkpoint AFTER extract (KV key)              ── one savepoint
   ├─ persist (relational + AGE + embeddings + fence)
   └─ complete | fail | retry whole task
```

Evidence:

- `task_impl.rs` dispatches by `TaskType`; each branch runs the full modality pipeline inside one `process()` call.
- `pipeline_checkpoint.rs` saves after expensive extraction; design text still assumes **KV** checkpoint keys and “single-worker per document.”
- `worker.rs` WHY-block: default `num_cpus * 4` because “pipeline is IO-bound” — true for *cloud LLM wait*, false for *PDF CPU* and *local Ollama serial*, and **always** conflates stages into one pool size.
- `local_inference_gate.rs` still wraps provider calls in `safety_limits.rs` — a **second** process-local budget beside the cluster ledger (LAW-Q3 residual).
- SPEC-120 value types (`Job`, `TaskAttempt`, `JobState`) exist in `job.rs` but are **storage-neutral / unwired** — the product still speaks `Task`, not Job→Stage.

| Concern | Today | Grade |
| --- | --- | --- |
| Unit of claim | Whole document pipeline | **Weak** vs A-WP2/A-WP3 |
| Provider slot grain | Coupled to call sites + residual local gate | **Partial** |
| Stage resume | One coarse checkpoint | **Partial** |
| Pool specialization | One `WorkerPool` for ingest+lifecycle types | **Weak** |
| CPU vs async isolation | PDF/parse on async workers | **Weak** (Tokio 100µs yield rule) |
| Bulk sliding window | Unbounded Pending + worker N | **Partial** (admission soft bound exists; no explicit window) |
| Status path isolation | Progress writes share ingest saturation | **Partial** |

### 4.3 Lens grades (summary)

| Lens | Grade | One-line |
| --- | --- | --- |
| L1 First principles | **Partial** | Control-plane axioms honored; execution-unit axioms not |
| L2 Code-is-law | **Strong** claim / **Weak** process body | 2.2kLOC worker is a symptom of accumulated cross-cutting concerns |
| L3 Systems eng. | **Partial** | Matches Postgres job-queue best practice; misses stage pools + sliding window |
| L4 Rust/Tokio | **Partial** | Good async claim loop; CPU work and mega-task topology fight the runtime |
| L5 Capacity | **Partial** | B is cluster-global; held across non-provider wall time / dual gates |
| L6 SOLID | **Weak** | `worker.rs` owns claim, fairness, admission, heartbeat, retry, park — SRP stress |

---

## 5. Alternatives considered (decision matrix)

| Option | Fits A-WP*? | Cost | Verdict for EdgeQuake |
| --- | --- | --- | --- |
| **A. Sync ingest in HTTP** | Violates A-WP1/A-WP5 | Low code, high ops pain | **Reject** — already abandoned correctly |
| **B. Keep monolithic document worker (status quo)** | Weak on A-WP2/A-WP3 | Lowest short-term | **Reject as end-state** — keep only as interim |
| **C. Pure in-process actors / mpsc pipelines (no durable claim)** | Violates A-WP1 | Medium | **Reject** as sole model — actors OK *inside* a claimed stage |
| **D. Kafka / Redpanda stream topology** | Possible | High ops; at-least-once still required | **Reject** unless multi-region fan-in measured |
| **E. External durable workflow (Temporal/Hatchet)** | Strong | New dependency, ops surface, payload limits → object store bus | **Defer** — adopt only if WP2–WP3 cannot deliver resume/isolation at measured scale |
| **F. Stage-bounded workers on Postgres claim (recommended)** | Strong | Medium; reuses claim/lease/budget | **Accept** — LD-18 |
| **G. Document orchestrator task + child stage tasks** | Strong | Slightly more rows | **Accept as F’s implementation shape** |

**Why not Temporal now?** EdgeQuake already owns Postgres, leases, heartbeats, cancel intents, provider slots, and a typed fence. Temporal’s value is *durable orchestration + activity isolation + sliding window*. Those **contracts** can be implemented on the existing claim table (Job + StageAttempt) without a second system of record — matching LD-03/LD-05 (ports and migrations own durability). Revisit E only after WP3 acceptance fails a documented soak (100k docs / multi-hour PDF) with evidence.

**Why workers remain correct:** Every serious option still *has* workers. The brand name is not the question; **the unit of work and the resource held while working** are.

---

## 6. Laws (`LAW-WP1..WP8`)

Specialize LAW-Q* / LAW-D* for **execution topology**:

| Law | Statement | Derives from |
| --- | --- | --- |
| **LAW-WP1** | **Control plane = durable claim.** Every ingest intent is a row claimable with `SKIP LOCKED` + lease + fencing; channels never authorize work. | A-WP1; LAW-Q2; SPEC-057 |
| **LAW-WP2** | **Execution unit = stage attempt, not document lifetime.** A worker claim covers one stage class (Prepare \| Extract \| Embed \| Materialize \| Lifecycle) with an idempotent commit or compensating fence. | A-WP2/A-WP3 |
| **LAW-WP3** | **Scarce leases match scarce work.** `provider_slot` is held only during provider-bound stages; parse CPU and DB materialize use their own budgets. Dual process-local gates are forbidden when the cluster ledger is enabled. | A-WP2; LAW-Q3; LD-11 |
| **LAW-WP4** | **Pools specialize by bottleneck.** At least two claim populations: *transform* (CPU/parse) and *infer* (LLM/embed); lifecycle (delete/wipe) never shares infer concurrency. Status/progress writes must not starve under ingest saturation. | A-WP2/A-WP5; Rapidflare isolation |
| **LAW-WP5** | **Tokio topology matches work class.** Async I/O on the ingest runtime; CPU-bound parse/PDF on `spawn_blocking` or a dedicated blocking pool; never block a serve runtime worker >~100µs without yield. | Tokio LTS practice 2026 |
| **LAW-WP6** | **Job is the user-visible SSOT; Task/StageAttempt is the scheduler SSOT.** UI/API progress projects from Job + structured `progress_counts`; workers mutate attempts through one transition table. | A-WP1; LAW-D4; LAW-IS1 |
| **LAW-WP7** | **Bulk fan-out is a sliding window.** In-flight stage attempts ≤ `f(B, parse_slots, embed_slots)`; never “admit 100k Pending and hope workers drain.” | Little’s Law; LAW-Q4 |
| **LAW-WP8** | **No second orchestrator without a failed gate.** External workflow engines are allowed only after WP-AC soak failure is recorded; until then Postgres claim is the orchestrator. | LD-03/05; cost of dual SoR |

---

## 7. Findings (`F-WP-01..18`)

| ID | Finding | Law |
| --- | --- | --- |
| F-WP-01 | Worker paradigm for **claim/lease** is correct and should be retained | LAW-WP1 |
| F-WP-02 | `process()` is a monolithic document saga inside one claim | LAW-WP2 |
| F-WP-03 | One `WORKER_THREADS` / pool size conflates parse, extract, embed, delete | LAW-WP4 |
| F-WP-04 | Provider budget can be shadowed by `local_inference_gate` | LAW-WP3 |
| F-WP-05 | Fairness park set + permit handoff + byte budget are process-local | LAW-WP3/Q3 |
| F-WP-06 | Checkpoint is single-boundary and still KV-shaped | LAW-WP2; LD-01 residual |
| F-WP-07 | PDF/CPU work can starve cooperative async workers | LAW-WP5 |
| F-WP-08 | `job.rs` Job/TaskAttempt types exist but are unwired | LAW-WP6 |
| F-WP-09 | No sliding window on bulk Pending depth vs B | LAW-WP7 |
| F-WP-10 | Lifecycle tasks share the same worker population as ingest | LAW-WP4 |
| F-WP-11 | Progress/status persistence can contend with saturated ingest | LAW-WP4 |
| F-WP-12 | Cancel gates are convention, not an enumerated stage-boundary suite | LAW-Q7 |
| F-WP-13 | Long-pole PDF holds a fairness/ingest slot while other docs wait | LAW-WP2/WP7 |
| F-WP-14 | Retry requeues the *whole* task after mid-pipeline failure | LAW-WP2; A-WP3 |
| F-WP-15 | Ingest `start_on` dedicated runtime exists but stage topology does not exploit it | LAW-WP5 |
| F-WP-16 | `worker.rs` SRP overload (~2.2k LOC) raises change risk | SOLID |
| F-WP-17 | Adopting Temporal now would duplicate SoR (tasks + workflow history) | LAW-WP8 |
| F-WP-18 | Actor/channel pipelines alone cannot replace durable claim | LAW-WP1 |

---

## 8. Target architecture

### 8.1 Locked decision

**LD-18 — Worker control plane retained; execution becomes stage-bounded on Postgres.**  
EdgeQuake keeps durable claim workers. The unit of claim becomes a **StageAttempt** (or a Task typed by stage class) owned by a **Job**. Provider slots are acquired only for infer stages. External workflow engines are out of scope until WP-AC soak fails (LAW-WP8).

### 8.2 Stage classes (minimal set)

| Class | Work | Scarce resource | Idempotent output |
| --- | --- | --- | --- |
| **Prepare** | Validate, PDF→markdown, normalize text | CPU / RAM / vision calls (vision ⇒ also provider) | Staged markdown + content hash |
| **Extract** | Chunk + entity/relationship LLM | Provider budget B | Extraction snapshot (typed, not KV) |
| **Embed** | Dense vectors | Provider / embed QPS | `chunk_embeddings` upsert |
| **Materialize** | AGE merge, CQRS, fence ready, outbox | DB write bandwidth | Fence `ready` iff tuple-complete |
| **Lifecycle** | Delete, wipe, compensate | DB + storage I/O | Tombstone / empty serving set |

Prepare may split **Parse** vs **Vision** later if measurement shows contention (OCP: add class without rewriting claim).

### 8.3 Topology

```ascii
                    ┌─ provider_slot ledger (B) ──────────────┐
                    │  acquired only by Extract / Embed /     │
                    │  Vision-Prepare                         │
                    └─────────────────────────────────────────┘
 Job (user SSOT)
   │
   ├─ StageAttempt Prepare ── claim: parse_pool
   │         │ success → enqueue Extract
   ├─ StageAttempt Extract ── claim: infer_pool + provider_slot
   │         │ success → enqueue Embed (or combined if measured cheaper)
   ├─ StageAttempt Embed ──── claim: infer_pool + provider_slot
   │         │ success → enqueue Materialize
   └─ StageAttempt Materialize ─ claim: materialize_pool (no provider)
             │ success → Job Succeeded; fence already set in same commit path
```

Intra-stage concurrency (chunk fan-out) remains **inside** a claimed stage via bounded semaphores derived from `ProviderProfile` (LAW-Q1) — actors/channels are fine *inside* the stage, never as the durability boundary.

### 8.4 Rust shape (practices)

1. **Split `worker.rs`:** `claim_loop`, `fairness_park`, `lease_heartbeat`, `pool_supervisor` — each ≤ ~400 LOC, tested independently.
2. **`TaskProcessor` → `StageProcessor`:** `async fn run(&self, attempt: &mut StageAttempt, cancel: CancellationToken)`.
3. **CPU:** PDF raster/parse via `tokio::task::spawn_blocking` (or dedicated blocking runtime); never on Axum’s serve runtime.
4. **Structured concurrency:** stage tasks use `CancellationToken` + RAII guards for lease/slot (already patterned by `HeartbeatGuard` / `ProviderSlotGuard`).
5. **No `unwrap` on hot paths; no holding `Mutex` across `.await`** in park/handoff maps — prefer `tokio::sync` or shard-by-track_id if maps remain.

---

## 9. Waves (`WP0`–`WP5`)

```
- [x] WP0 — Measure & freeze laws (no behavior change)
- [x] WP1 — Stage-boundary resource discipline (inside monolith)
- [ ] WP2 — StageAttempt claim + dual pools (Prepare/Lifecycle vs Infer/Materialize)
- [ ] WP3 — Job SSOT + sliding window + typed stage checkpoints
- [ ] WP4 — Tokio topology hardening (blocking pool, status isolation)
- [ ] WP5 — Retire dual gates; conformance + soak; LAW-WP8 go/no-go
```

### WP0 — Measure & freeze (entry: always)

| Deliverable | Detail |
| --- | --- |
| Metrics | Per-stage wall time, provider_slot hold time, worker busy ratio, park rate, claim latency |
| Baseline | 10 / 100 / 1k doc suites; one multi-hundred-page PDF long-pole |
| Doc | This file + README index; LD-18 recorded |
| Exit | Dashboard/query or tracing fields prove F-WP-02/13 (slot hold ≫ infer time) |

### WP1 — Resource discipline without re-queue (entry: WP0)

Keep document `Task`, but:

1. Acquire `provider_slot` only around extract/embed/vision call scopes; release between stages.
2. Do not hold fairness ingest permit across pure DB materialize if permit is process-local — or migrate permit to stage scope.
3. Add cancel-gate conformance list (claim, pre-extract, pre-embed, pre-materialize, park).
4. Move checkpoint write path toward typed storage (document-scoped row / jsonb on `documents` or `ingestion_stage_state`) — dual-read if needed (LD-07).

Exit: F-WP-04 mitigated (local gate no longer raises effective B); slot occupancy metric drops toward infer time.

### WP2 — StageAttempt + specialized pools (entry: WP1)

1. Persist `StageAttempt` (reuse `TaskAttempt` / extend `tasks` with `stage_class` — one migration-owned change, LD-03).
2. Supervisor enqueues next stage on success; failure retries **stage** with classified backoff.
3. Two pools minimum: `transform_pool` (Prepare+Lifecycle) and `infer_pool` (Extract+Embed); Materialize can share transform or get its own if write contention measured.
4. Split processor modules by stage (SRP).

Exit: dual-pool contract test — embed saturation cannot block delete; parse CPU cannot multiply provider calls.

### WP3 — Job SSOT + sliding window (entry: WP2)

1. Wire `Job` / `JobState` as API+UI authority (LAW-WP6); tasks become attempts.
2. Admission: soft Pending bound already (LD-12); add **in-flight window** `W = f(B, parse_slots)` so bulk uploads do not create unbounded Processing/Pending storms.
3. Typed stage checkpoints replace KV checkpoint keys for new writes.
4. Long-pole PDF: optional page/chunk child attempts under the same Job (measurement-gated).

Exit: 1k-doc upload respects window; ETA honest (LAW-Q4); UI reads Job projection.

### WP4 — Tokio hardening (entry: WP2, parallelizable with WP3)

1. PDF/parse on blocking pool; document policy in `safety_limits` / processor.
2. Optional low-concurrency **status writer** path (dedicated permits) so progress updates never wait behind extract.
3. Keep ingest on `start_on` dedicated runtime; serve runtime remains latency-critical.

Exit: serve `/health` p99 unchanged under ingest soak; no cooperative-starvation incidents in chaos.

### WP5 — Closure & go/no-go (entry: WP3+WP4)

1. Remove or hard-wire `local_inference_gate` to `ProviderBudget` (one SSOT).
2. Cluster-ize or eliminate process-local park set (DB park marker already exists — make it sufficient).
3. Conformance suite: multi-replica stage claim, cancel at each boundary, lease-loss mid-stage, compensate+fence.
4. Soak: kill-9 mid-stage; 100k synthetic; one multi-hour PDF corpus.
5. **LAW-WP8 decision record:** if soak fails on orchestration complexity (not on data model), open Temporal spike with explicit dual-SoR mitigation plan; else close “workers+stages on Postgres” as permanent.

---

## 10. Acceptance (`WP-AC-01..14`)

| ID | Gate | Wave |
| --- | --- | --- |
| WP-AC-01 | Written LD-18 + laws; README indexes doc 24 | WP0 **Met** |
| WP-AC-02 | Metrics distinguish stage wall vs provider_slot hold | WP0 **Met** |
| WP-AC-03 | With ledger enabled, local gate cannot exceed B | WP1 **Met** |
| WP-AC-04 | Cancel conformance enumerates all stage boundaries | WP1 **Met** |
| WP-AC-05 | New checkpoints are typed (no new KV checkpoint keys) | WP1 **Met** (relational + typed success ⇒ KV write-stop) |
| WP-AC-06 | StageAttempt is the claim row; document Job owns graph | WP2 |
| WP-AC-07 | Infer pool saturation ≠ blocked Lifecycle claim | WP2 |
| WP-AC-08 | Retry after extract failure does not re-run Prepare when snapshot valid | WP2 |
| WP-AC-09 | Bulk admit respects sliding window W=f(B,…) | WP3 |
| WP-AC-10 | API/UI Job projection is progress SSOT (with `progress_counts`) | WP3 |
| WP-AC-11 | PDF CPU off serve/ingest async worker threads | WP4 |
| WP-AC-12 | `/health` p99 within budget during ingest soak | WP4 |
| WP-AC-13 | Multi-replica stage claim: exactly one runner per attempt | WP5 |
| WP-AC-14 | Kill-9 mid-stage resumes without duplicate fence-ready tuples | WP5 |

---

## 11. Risks & edge cases

| ID | Risk / edge | Mitigation |
| --- | --- | --- |
| R-WP-01 | Stage split increases row churn / claim latency | Batch claim; partial indexes on `(stage_class, status, created_at)`; measure in WP0 |
| R-WP-02 | Dual-write Job+Task during migration drifts | Single writer module; transition table tests; LD-07 flag |
| R-WP-03 | Vision-Prepare needs provider_slot — class bleed | Vision sub-class under Prepare with LAW-WP3 acquire |
| R-WP-04 | Sliding window feels like “uploads stuck” | Surface queue_position/ETA (doc 20 IS2); never silent hang (LD-12) |
| R-WP-05 | Over-splitting Embed vs Extract adds RTT | Allow combined Infer stage when B small (local GPU); profile-gated |
| R-WP-06 | Temporal temptation mid-flight | LAW-WP8; only after WP-AC-14 failure |
| EC-WP-01 | Lease expiry mid-Materialize after AGE write | Fence fail-closed until compensate; CAS on attempt token |
| EC-WP-02 | Reprocess while StageAttempts in flight | Job generation/fence epoch; supersede old attempts |
| EC-WP-03 | Delete during Extract | Cancel intent preemptive at every stage enqueue/claim (LAW-Q7) |

---

## 12. Mapping to prior SPEC-091 work

| Prior | Relationship |
| --- | --- |
| LAW-Q1..Q7 / LD-11..13 | Preserved; WP makes resource grain match Q axioms |
| Doc 21 IP3–IP5 | Orthogonal data-model residuals; WP does not replace them |
| Doc 22 RM* | Outbox drain remains; stage Materialize is the natural outbox producer |
| Doc 20 IS2–IS3 | Job SSOT (WP3) unblocks honest queue chrome |
| Doc 23 KVH* | Stage checkpoints must not reintroduce KV hot paths |
| SPEC-120 job types | Activated by WP2–WP3 instead of a parallel product |

---

## 13. What this deliberately does NOT change

- Serving fence semantics (LD-09) or typed chunk identity (LD-01/02).
- AGE as traversal authority (LD-04).
- Wake-channel-as-wake-only design.
- HTTP admit → 202 contract.
- Migration engine / boot refuse (LD-15).

---

## 14. Implementation sketch (ordering discipline)

1. **Instrument before split** (WP0) — otherwise stage boundaries are taste.
2. **Release slots before re-queueing** (WP1) — cheapest win, proves LAW-WP3.
3. **Add StageAttempt beside Task** (WP2) — dual-read; flip claim filter by flag.
4. **Promote Job to API** (WP3) — FE already wants one operation narrative.
5. **Only then** consider child page attempts / Temporal (WP5 evidence).

---

## 15. References (July 2026)

- PostgreSQL `FOR UPDATE SKIP LOCKED` job queues — [Prisma: You don’t need a job queue](https://www.prisma.io/blog/you-dont-need-a-job-queue-postgres-already-has-skip-locked); lease + claim_token fencing patterns.
- Production RAG ingestion control plane — [ToolLeap: Workers before GPUs](https://blog.toolleap.com/production-rag-ingestion-pipeline/) (durable jobs, stage idempotency, per-stage backpressure, publish ≠ parse).
- Stage-isolated workers + sliding window — [Rapidflare on Temporal](https://temporal.io/blog/how-rapidflare-built-a-million-document-ingestion-pipeline-for-agents-on-temporal) (contracts to copy; engine optional).
- Tokio — cooperative scheduling, `spawn_blocking`, separate runtimes for CPU vs latency; prefer short tasks + bounded channels between stages ([Async Rust as architecture](https://medium.com/rustaceans/async-rust-as-an-architectural-pattern-not-just-async-fn-cc43a2bede36); InfluxData on CPU-bound Tokio).
- In-tree: `worker.rs`, `state_machine.rs`, `provider_budget.rs`, `pipeline_checkpoint.rs`, `job.rs`, docs [12](12-queue-admission-first-principles.md)–[14](14-queue-admission-plan.md).
