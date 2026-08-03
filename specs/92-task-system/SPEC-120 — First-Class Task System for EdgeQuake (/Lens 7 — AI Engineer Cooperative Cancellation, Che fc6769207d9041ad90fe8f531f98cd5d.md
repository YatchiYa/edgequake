# Lens 7 — AI Engineer: Cooperative Cancellation, Checkpoints, Cost-Weighted Fairness

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). Normative for behaviour inside the model pipeline: where cancellation is observed, what a partial result costs, and how consumed capacity is measured. Scheduling belongs to Lens 4, timing budgets to Lens 5.
> 

## Owning the term that makes cancellation slow

Lens 5 budgets "≤ 4 s to reach an await point". That term is entirely determined by this layer. Cancellation in Rust is cooperative: a token can only take effect where the future yields. A conversion stage that issues one 300-second vision call and checks the token only before and after it cannot be cancelled inside that window, no matter how good the durable intent mechanism is.

```
INGEST PIPELINE, WITH CANCELLATION OBSERVABILITY MARKED

  stage            typical duration   check points today   required
  ───────────────────────────────────────────────────────────
  fetch / stage    1–10 s             boundary             boundary + stream
  convert (vision) 10 s – 20 min      boundary             per page
  extract          5–60 s             boundary             per document part
  chunk            < 1 s              boundary             boundary
  embed            5 s – 5 min        boundary             per batch
  index vectors    1–30 s             boundary             per batch + fence
  index graph      1–60 s             boundary             per batch + fence
  index kv         < 1 s              boundary             boundary + fence
  finalise         < 1 s              boundary             fence

  RULE: no uninterruptible span longer than 5 seconds may exist between two
  cancellation checks, and every persist call must carry the fence epoch.
```

The branch already understands the shape of this problem: `vision_stall_watchdog.rs` exists precisely because a vision call can stop making progress, and `TaskFailureInfo::with_made_progress` plus the `[vision_progress=N]` marker prevent a slow-but-advancing conversion from tripping the circuit breaker. Those mechanisms measure progress; this lens asks them to also *observe intent* at the same granularity.

## Placing the checks

```rust
// The unit of work becomes a loop over checkpointable items rather than one
// long await. Cancellation, heartbeating, and fencing share the same tick.
async fn convert(lease: &Lease<Running>, doc: &Source) -> Result<Outcome> {
    for page in doc.pages() {
        match lease.heartbeat().await? {          // LeaseVerdict, see Lens 5
            LeaseVerdict::Renewed => {}
            LeaseVerdict::Lost => return Ok(Outcome::Abandoned),
            LeaseVerdict::CancelRequested => return Ok(Outcome::Draining),
        }

        // Race the provider call against the token so an in-flight HTTP request
        // is dropped rather than awaited to completion.
        let text = tokio::select! {
            r = vision.describe(page) => r?,
            _ = lease.cancelled()    => return Ok(Outcome::Draining),
        };

        lease.persist(PageText { page: page.index, text }).await?;  // fenced
        lease.checkpoint(page.index).await?;                         // resumable
    }
    Ok(Outcome::Completed)
}
```

Three properties follow from this shape. The provider call is abandoned rather than awaited, which is what actually saves money. Every persist is epoch-conditional, so a straggler cannot resurrect data after a delete (hub gap G2). And the checkpoint makes the work resumable, which changes the economics of retry: a 900-page conversion that fails at page 880 resumes at 880 rather than at 1.

### What must never be cancelled

```
CANCELLABLE                          NOT CANCELLABLE (must run to completion)
───────────────────────────────────────────────────────────────────
provider inference calls             a single vector upsert batch in flight
chunking and embedding loops         a graph transaction already opened
fetching source bytes                the compensation that retracts indexes
waiting for a fairness permit        writing the terminal state row

A cancellation that interrupts compensation converts a clean stop into an
inconsistent store. Compensation runs with cancellation masked, and its own
failure path is the quarantine record, not another cancel.
```

This is the discipline behind `cancel_retract.rs` and `text_insert/cancel.rs::retract_indexes_on_cancel`, whose `is_post_graph_stage` check decides how much needs undoing. Keep that logic; give it an explicit no-cancel scope so it cannot be interrupted halfway.

## Making partial work cheap to undo

```
WRITE ORDER AND UNDO ORDER ARE MIRRORS

  write:  chunks → vectors → graph nodes → graph edges → kv summary → doc status
  undo :  doc status ← kv summary ← graph edges ← graph nodes ← vectors ← chunks

  Each undo step must be idempotent and keyed by (document_id, fence_epoch),
  so running it twice is harmless and running it after a partial failure resumes.

  ┌─────────────────────────────────────────────────────────┐
  │ cancel at stage k → undo stages k…1 in reverse, then mark cancelled  │
  │ undo failure      → dead_letter + compensation_quarantine:{doc}:*    │
  │                     (the existing KV dead-letter convention)         │
  └─────────────────────────────────────────────────────────┘
```

The cheaper alternative where the store supports it: write everything under the epoch and make visibility conditional on the document's current epoch. Then "undo" is a single epoch bump and a background sweep, and cancellation cost becomes constant rather than proportional to work done. Vector stores that support metadata filtering can do this today; treat it as the target and reverse-order compensation as the fallback.

## Measuring cost, not counting tasks

The fairness ledger in Lens 4 charges `vruntime` by consumed capacity. This lens defines the estimator, because slot counting systematically misprices exactly the workloads that hurt.

```
COST MODEL (normalised units, calibrated per deployment)

  convert   cost = pages × vision_unit
                   vision_unit ≈ GPU-seconds or provider price per page
  extract   cost = input_tokens × extract_unit
  embed     cost = chunks × embed_unit
  index     cost = vectors_written × index_unit
  delete    cost = estimated_rows_touched × purge_unit

  ESTIMATE BEFORE, CHARGE AFTER
    at enqueue: cost_estimate from file size, page count, byte size
                (the same signals estimate_task_bytes already reads:
                 file_size, size_bytes, byte_size, content_length, bytes)
    at finish : charge the measured cost, not the estimate
    on cancel : charge what was consumed up to the stop point — a tenant that
                repeatedly starts and cancels expensive work still pays for it

  WHY IT MATTERS
    tenant A: 2 × 900-page scanned PDFs        ≈ 1800 vision units
    tenant B: 2 × 1-page text notes            ≈ 2 embed units
    Under slot fairness these are equal. Under cost fairness B is served
    hundreds of times more often, which is the intuitively correct answer.
```

Charging cancelled work is the anti-abuse property: without it, cancellation becomes a free way to consume provider capacity.

## Budgeting per tenant, not just limiting concurrency

```
┌───────────────────────────────────────────────────────────┐
│ GATE ORDER AT CLAIM TIME                                            │
│                                                                     │
│  1. quota      active(tenant,lane) < max_concurrent   ← Lens 4      │
│  2. fair share lowest vruntime / weight               ← Lens 4      │
│  3. bytes      InFlightByteBudget admits              ← admission.rs │
│  4. tokens     tenant token budget for the window     ← NEW         │
│  5. provider   provider-level rate limit not tripped  ← NEW         │
│                                                                     │
│  Gates 4 and 5 exist because the scarce resource for AI work is not  │
│  a worker slot or a megabyte; it is provider throughput. A tenant    │
│  within its slot quota can still exhaust a shared model endpoint     │
│  and thereby starve every other tenant.                             │
└───────────────────────────────────────────────────────────┘
```

Provider selection already has a documented precedence chain (`EDGEQUAKE_EXTRACT_PROVIDER` → `EDGEQUAKE_DEFAULT_EXTRACT_PROVIDER` → `EDGEQUAKE_DEFAULT_LLM_PROVIDER` → `EDGEQUAKE_LLM_PROVIDER`). Each resolved provider needs its own concurrency and rate ledger, because a local Ollama endpoint and a hosted API have different scarcity profiles — which is exactly why `WORKER_THREADS` is capped at four for local providers unless `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1`.

## Timing out with evidence

```
STALL versus SLOW

slow  : progress markers advance, no output yet  → extend, do not fail
stall : no progress marker for T seconds         → fail as timeout

timeout source of truth: metadata.processing_timeout_secs, derived from
  LargeDocumentProfile::{convert_timeout_secs, ingest_timeout_secs}

breaker: 3 consecutive typed timeouts → circuit_breaker_tripped
         with_made_progress(true) does NOT advance the breaker

ADD: progress-derived timeouts per checkpoint rather than per task, so a
900-page document is not given the same budget as a 2-page one, and a
document that stalls at page 3 fails in seconds rather than in minutes.
```

This distinction is already present in the branch and is one of its better ideas; the change is to evaluate it per checkpoint, which is only possible once checkpoints exist.

## Reporting progress that can be trusted

| Rule | Reason |
| --- | --- |
| Progress is monotonic per attempt, and resets visibly on a new attempt | a bar that goes backwards without explanation reads as a bug |
| Report stage plus items done over items total, never a synthesised percentage | the interface can decide how to render uncertainty (Lens 8) |
| Emit progress as append-only events, never as updates to the task row | avoids the write amplification described in Lens 4 |
| Unknown totals are reported as unknown | for a scanned document the page count may not be known until conversion starts |
| Cancellation is a first-class event, not the absence of progress | the interface needs to distinguish stopping from stalled |

## Where to read next

Fence semantics and the compensation ordering guarantees are in Lens 3. The ledger that consumes this cost model is in Lens 4. The cancellation time budget this lens must satisfy is in Lens 5. Typestate that forces fenced persistence is in Lens 6. Progress rendering and uncertainty are in Lens 8.