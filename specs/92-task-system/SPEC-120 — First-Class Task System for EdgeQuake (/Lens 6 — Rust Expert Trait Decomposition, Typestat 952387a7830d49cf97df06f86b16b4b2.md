# Lens 6 — Rust Expert: Trait Decomposition, Typestate, DRY and SOLID Refactors

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). Normative for crate structure, trait boundaries, and type-level guarantees inside `edgequake-tasks` and the API services. The behaviour these types must express is defined in Lens 3 and Lens 5.
> 

## Crediting the existing design

Three things in this crate are already idiomatic and should survive the refactor untouched in spirit. Permits are RAII (`FairnessPermit`, `AdmissionPermit`, `ParkWaitGuard` with `arm_success`), so capacity is released on every exit path including panics and cancellation. Outcomes are typed rather than boolean (`TryAcquireOutcome::{Unlimited, Acquired, AtCapacity}`, `AdmissionOutcome::{Admitted, Rejected{..}}`), so a caller cannot confuse "no limit configured" with "limit available". And dependency inversion is real: the API services depend on `SharedTaskStorage` and `CancellationRegistry`, never on Postgres types.

What follows fixes the boundaries around that good core.

## Splitting the fat port

`TaskStorage` carries roughly twenty methods across eight responsibilities. Every consumer depends on all of them, every test double must implement all of them, and PDF domain knowledge leaks into a generic persistence abstraction.

```
BEFORE                                AFTER
┌─────────────────────────┐          ┌──────────────────┐
│ trait TaskStorage      │          │ TaskRepository   │ create, get, update, list
│                        │          └──────────────────┘
│ create_task            │          ┌──────────────────┐
│ get_task               │          │ TaskClaimer      │ claim_next_with_policy
│ update_task            │          └──────────────────┘
│ delete_task            │          ┌──────────────────┐
│ list_tasks             │          │ LeaseKeeper      │ refresh, release
│ update_task_progress   │          └──────────────────┘
│ touch_task             │          ┌──────────────────┐
│ claim_next             │          │ FairnessLedger   │ hold, clear, charge, quota
│ claim_next_with_policy │          └──────────────────┘
│ refresh_lease          │          ┌──────────────────┐
│ release_claim          │          │ CancelStore      │ request_cancel, poll_intent
│ mark_fairness_hold     │          └──────────────────┘
│ clear_fairness_hold    │          ┌──────────────────┐
│ get_statistics         │          │ MetricsReader    │ queue metrics, statistics
│ get_queue_metrics      │          └──────────────────┘
│ get_queue_metrics_filt │          ┌──────────────────┐
│ prune_terminal_tasks   │          │ TaskAdmin        │ prune, partitions
│ ensure_month_partitions│          └──────────────────┘
│ find_active_pdf_proc…  │          ┌──────────────────┐
│ find_active_pdf_ingest │  ◄ domain │ PdfTaskQueries   │ built ON TaskRepository,
└─────────────────────────┘    leak    └──────────────────┘ not a peer of it

One Postgres struct still implements all seven ports, so nothing is harder to
wire; but the worker depends on TaskClaimer + LeaseKeeper, the cancel facade on
CancelStore + TaskRepository, and a test double implements only what it needs.
```

### Removing the duplicated finders

`find_active_pdf_processing_task` and `find_active_pdf_ingest_task` are the same nested paging loop over two task types and two statuses. That is the clearest DRY violation in the crate, and it disappears once the identifiers are real columns (Lens 4):

```rust
pub struct ActiveTaskQuery {
    pub operations: SmallVec<[Operation; 4]>,
    pub states:     SmallVec<[TaskState; 4]>,
    pub scope:      TaskScope,          // Document(Uuid) | Pdf(Uuid) | Workspace(Uuid)
}

// One method replaces both finders and every future variant of them.
async fn find_active(&self, q: &ActiveTaskQuery) -> Result<Vec<Task>>;

// PdfTaskQueries becomes a thin, testable composition instead of a port method.
impl<R: TaskRepository> PdfTaskQueries for R {
    async fn active_convert(&self, pdf: Uuid) -> Result<Option<Task>> {
        self.find_active(&ActiveTaskQuery::convert_for_pdf(pdf)).await.map(one)
    }
}
```

## Making illegal states unrepresentable

### Newtypes for identifiers

```rust
// Today: track_id, worker_id, document_id, pdf_id are all String or Uuid,
// interchangeable at every call site. The compiler cannot help.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct TrackId(String);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct TaskId(Uuid);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct DocumentId(Uuid);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct LeaseToken(Uuid);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct FenceEpoch(i64);
```

The payoff is concrete: `refresh_lease(track_id, worker_id, lease_token, ttl)` currently takes two strings and a UUID in a fixed order, and a transposed pair compiles cleanly while silently failing to renew.

### One transition table, no scattered guards

```rust
pub enum TaskState { Queued, Held, Leased, Running, Cancelling,
                     Succeeded, Failed, Cancelled, DeadLetter }

impl TaskState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Cancelled | Self::DeadLetter)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("illegal transition {from:?} -> {to:?}")]
pub struct IllegalTransition { pub from: TaskState, pub to: TaskState }

/// THE single source of truth for legality. Nothing else may write `state`.
pub const fn allows(from: TaskState, to: TaskState) -> bool {
    use TaskState::*;
    match (from, to) {
        (Queued,  Held | Leased | Cancelling | Cancelled) => true,
        (Held,    Queued | Leased | Cancelling | Cancelled) => true,
        (Leased,  Queued | Running | Cancelling | Failed | Cancelled) => true,
        (Running, Queued | Cancelling | Succeeded | Failed) => true,
        (Cancelling, Cancelled) => true,
        (Failed,  Queued | Cancelled | DeadLetter) => true,
        (DeadLetter, Queued) => true,                 // explicit operator retry
        _ => false,
    }
}

#[must_use = "an ignored transition result silently drops a state change"]
pub fn try_transition(t: &mut Task, to: TaskState)
    -> Result<TransitionEvent, IllegalTransition> { /* … */ }
```

Today `mark_success`, `mark_failed_with_details`, and `mark_cancelled` each carry their own guard, and `mark_success` returns a bare `bool` that a caller may ignore. Replacing them with one function that returns a `#[must_use]` result gives three benefits at once: the matrix becomes exhaustively testable by iterating the cartesian product, the same table can generate the SQL constraint from Lens 3, and adding a state produces compile errors at exactly the places that need thought.

### Typestate for the execution path

```rust
pub struct Claimed;  pub struct Running;  pub struct Draining;

pub struct Lease<S> { task: TaskId, token: LeaseToken, epoch: FenceEpoch,
                      _s: PhantomData<S> }

impl Lease<Claimed> {
    pub fn begin(self) -> Lease<Running> { /* writes the attempt row */ }
}
impl Lease<Running> {
    /// Only a Running lease can persist side effects, and only with its epoch.
    pub async fn persist(&self, w: impl FencedWrite) -> Result<()> { /* … */ }
    pub async fn heartbeat(&self) -> Result<LeaseVerdict> { /* … */ }
    pub fn drain(self) -> Lease<Draining> { /* … */ }
}
```

This encodes hub axiom 3 in the type system: a write requires a `Lease<Running>`, a `Lease<Running>` carries a `FenceEpoch`, and there is no constructor that produces one without a claim. The resurrection class of bug becomes a compile error rather than a race.

## Repairing the concurrency primitives

### The lane key must be a real hash

```rust
// Today — comment says "unique enough for semaphore map keys in-process":
//   Uuid::from_u128(t ^ w.rotate_left(17) ^ 0x0840_0316_u128)
// XOR of two UUIDs is not injective: distinct (tenant, workspace) pairs collide,
// and colliding pairs silently share one concurrency lane.

const LANE_NS: Uuid = uuid!("b2c9a1f4-0000-0000-0000-000000000000");

pub fn lane_key(tenant: TenantId, workspace: WorkspaceId, class: FairnessClass) -> Uuid {
    let mut buf = [0u8; 33];
    buf[..16].copy_from_slice(tenant.as_bytes());
    buf[16..32].copy_from_slice(workspace.as_bytes());
    buf[32] = class as u8;
    Uuid::new_v5(&LANE_NS, &buf)
}
```

Including the class in the key also removes a latent bug: ingest and lifecycle lanes are separate maps today, so the key alone does not identify a lane, and any future unification would silently merge them.

### The handoff map must expire

```rust
// Today: handoffs: Arc<Mutex<HashMap<String, FairnessPermit>>>
// A staged permit is only released if someone calls take_handoff with the exact
// track_id. Process death, a lost wake, or a sibling replica winning the claim
// leaks it, and the lane shrinks with no diagnostic.

struct StagedPermit { permit: FairnessPermit, staged_at: Instant }

pub struct Handoffs {
    inner: DashMap<TrackId, StagedPermit>,   // no global mutex on the hot path
    ttl: Duration,                            // 2 × claim interval
}

impl Handoffs {
    pub fn take(&self, id: &TrackId) -> Option<FairnessPermit> { /* … */ }
    /// Called from the same periodic tick that drives claiming.
    pub fn reap(&self) -> usize { /* drop entries older than ttl, return count */ }
}
```

The reaped count becomes a metric: a non-zero steady-state value means wakes are being lost, which is a signal the current design cannot produce.

### The queue trait must stop pretending

```rust
// Today: trait TaskQueue { send, receive, try_receive, size, is_closed }
// The documentation says delivery is Postgres and the channel is a wake signal,
// yet the trait's shape invites processing straight from the payload.

pub trait WakeSignal: Send + Sync {
    fn wake(&self, hint: WakeHint);              // fire and forget, never blocks
    async fn wait(&self, timeout: Duration) -> WakeReason; // Signalled | Timeout
}

pub enum WakeHint { Any, Tenant(TenantId), Cancel(TrackId) }
```

No payload, so a worker structurally cannot bypass the claim; `wait` always takes a timeout, so the periodic tick that guarantees correctness is part of the interface rather than a convention. `UnboundedChannelTaskQueue` is deleted rather than ported: an unbounded wake queue can only hoard duplicate wakeups.

## Testing the ports rather than the implementations

```rust
// One macro, run against Postgres and the in-memory double, closes the Liskov
// gap where memory.rs default methods scan while postgres.rs queries.
macro_rules! task_port_contract {
    ($name:ident, $ctor:expr) => {
        mod $name {
            #[tokio::test] async fn claim_is_exclusive_under_contention() { /* … */ }
            #[tokio::test] async fn cancelled_task_is_never_claimed() { /* … */ }
            #[tokio::test] async fn expired_lease_is_reclaimable_once() { /* … */ }
            #[tokio::test] async fn held_task_is_invisible_until_hold_expires() { /* … */ }
            #[tokio::test] async fn heartbeat_reports_cancel_intent() { /* … */ }
            #[tokio::test] async fn fenced_write_is_refused() { /* … */ }
        }
    };
}
task_port_contract!(pg,  PostgresTaskStore::for_test());
task_port_contract!(mem, MemoryTaskStore::for_test());

#[test]
fn transition_matrix_is_exhaustive_and_terminal_states_absorb() {
    for from in TaskState::ALL {
        for to in TaskState::ALL {
            if from.is_terminal() && from != to {
                assert!(!allows(from, to) || (from == DeadLetter && to == Queued));
            }
        }
    }
}
```

Also worth doing while the seams are open: make time and wakeups injectable (`trait Clock`, `trait Notifier`) so the drain barrier and hold expiry can be tested deterministically instead of with `sleep`, and put the in-memory store behind a `cfg(feature = "testing")` gate so it can never be selected in production by configuration accident.

## Mapping SOLID to concrete edits

| Principle | Edit |
| --- | --- |
| Single responsibility | seven ports replace one; `Task` loses its `mark_*` guards to the transition module |
| Open–closed | one `OperationDescriptor` registry supplies fairness class, timeout, and label; adding an operation touches one table, not five matches |
| Liskov | the `task_port_contract!` macro is mandatory for every implementation |
| Interface segregation | worker takes `TaskClaimer + LeaseKeeper`; cancel facade takes `CancelStore + TaskRepository` |
| Dependency inversion | add `Clock` and `Notifier`; keep the existing `Shared*` handles |
| DRY | one `find_active`, one transition table, one lane key, one status vocabulary shared with SQL and OpenAPI |

## Where to read next

The state names and fence semantics are in Lens 3, the SQL these ports issue is in Lens 4, and `LeaseVerdict` timing is in Lens 5. Where the cancellation checks must be placed inside the model pipeline is in Lens 7. The contract that these types serialise into is in Lens 2.