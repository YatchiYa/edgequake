# Lens 1 — Product Owner: Operations, Invariants, Roadmap

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). This lens is normative for the operation taxonomy, the user-visible guarantees, and the release sequencing. It defers to Lens 3 for storage shape, Lens 4 for claim behaviour, and Lens 8 for wording.
> 

## Stating the product thesis

EdgeQuake asks users to hand over documents and wait. Everything the user does after that — watching, cancelling, retrying, deleting, reprocessing — is a *task interaction*. Today that interaction is spread across ingestion endpoints, a PDF pipeline, a document deletion cascade, and a wipe path, each with its own vocabulary. A first-class task system means one noun (`operation`), one lifecycle, one place to look, and one promise per verb.

The promise, stated in one sentence: **any operation a user can start, the user can stop; stopping is honoured everywhere within seconds; and no tenant can make another tenant wait indefinitely.**

## Naming the operations

The branch has nine `TaskType` variants, of which two lifecycle verbs are typed and one important verb — reprocess — is not typed at all (hub gap G12).

| Operation | Today | Fairness class | Cancellable | Must cancel dependents first |
| --- | --- | --- | --- | --- |
| `ingest.upload` | `TaskType::Upload` | Ingest | yes | no |
| `ingest.insert` | `TaskType::Insert` | Ingest | yes | no |
| `ingest.convert` | `TaskType::PdfProcessing` | Ingest | yes | cancels child insert |
| `ingest.scan` | `TaskType::Scan` | Ingest | yes | no |
| `knowledge.inject` | `TaskType::KnowledgeInjection` | Ingest | yes | no |
| `document.reindex` | `TaskType::Reindex` | Ingest | yes | no |
| **`document.reprocess`** | **missing — reuses Insert/PdfProcessing** | Ingest | yes | cancels prior pipeline |
| `document.delete` | `TaskType::Deletion` | Lifecycle | yes, until fenced | **yes** |
| `document.delete_batch` | `TaskType::BatchDeletion` | Lifecycle | yes, until fenced | **yes** |
| `workspace.wipe` | `TaskType::WorkspaceWipe` | Lifecycle | yes, until fenced | **yes** |

Two product decisions follow. First, `document.reprocess` becomes a named operation so that a user who reprocesses a document can see it as such, and so that reprocess volume is separable in fairness accounting from first-time ingestion. Second, destructive operations become cancellable **only before the fence** — after the fence epoch is bumped, the deletion is a commitment and the interface must say so rather than offering a stop button that cannot work.

## Modelling the jobs to be done

```
PERSONA               JOB                                 TODAY            TARGET
─────────────────────────────────────────────────────────────────────────
Knowledge worker      "I uploaded the wrong file,
                       stop it now"                      partial          ≤ 5 s stop
                      "Delete it and be sure it is
                       gone"                             best effort      verified purge
                      "Why is nothing happening?"        no answer        queue position
Platform operator     "One tenant is flooding the
                       queue"                            manual           weighted share
                      "Which operations are stuck?"      per-replica      global view
                      "Replay what failed"               ad hoc           dead-letter list
Compliance owner      "Prove the document is deleted"     no artefact      purge receipt
```

The third row of each block is the honest reason this programme exists: the current system can already *usually* do the happy path, and cannot yet *prove* anything.

## Writing the invariants as acceptance criteria

These are the criteria the hub invariants translate into. They are written to be automatable.

```gherkin
Feature: Cancellation is a promise, not a request

  Scenario: Cancel reaches a task running on another replica
    Given two API replicas A and B share one Postgres
    And an ingest task T is running on replica B
    When a user cancels T through replica A
    Then T.cancel_requested_at is set within one round trip
    And replica B stops executing T within one heartbeat interval
    And the user-visible status becomes "Stopping" immediately
    And it becomes "Cancelled" only after B acknowledges the stop

  Scenario: Cancelled work leaves no residue
    Given an ingest task T for document D has written some vectors
    When T is cancelled
    Then all vectors, graph edges and key-value entries for D are retracted
    And any write attempted by T after cancellation is rejected by the fence

Feature: Deletion cancels first

  Scenario: Delete waits for a live ingest to stand down
    Given an ingest task T for document D holds a valid lease
    When a user deletes D
    Then a delete job is created in state REQUESTED
    And T receives a cancellation request before any purge step runs
    And no purge step runs while T still holds a valid lease
    And the delete job reaches DONE only after a zero-residue verification

  Scenario: Delete is cheap in a large workspace
    Given a workspace with one million task rows
    When a user deletes one document
    Then the dependent-task lookup touches only rows for that document
    And end-to-end latency is within the same order as a workspace of one thousand rows

Feature: Fairness between tenants

  Scenario: A flooding tenant cannot starve a quiet tenant
    Given tenant X enqueues one thousand ingests
    And tenant Y enqueues one ingest one second later
    When workers have free capacity for one task
    Then tenant Y's task starts before tenant X's tenth task
    And tenant Y's queue wait is recorded in the fairness metrics

  Scenario: Weight is respected, not just count
    Given tenant X submits documents that cost ten times more to convert
    When both tenants have equal weight
    Then the consumed capacity converges to equal shares, not equal task counts
```

## Sequencing the releases

```
P0 ── TRUST                 P1 ── SCALE               P2 ── FAIRNESS
┌──────────────────┐        ┌────────────────┐      ┌────────────────┐
│ durable cancel   │        │ indexed lookups │      │ durable ledger │
│ fence epoch      │  ──►   │ richer states   │ ──►  │ weights + DRR  │
│ delete barrier   │        │ backoff + DLQ   │      │ job graph      │
│ keep the row     │        │ reprocess type  │      │ port split     │
└──────────────────┘        └────────────────┘      └────────────────┘
ships correctness          ships headroom         ships multi-tenancy
measurable by INV-1..3     INV-7                  INV-5

                               P3 ── TRANSPARENCY
                               ┌───────────────────┐
                               │ global metrics     │
                               │ queue position UI  │
                               │ purge receipts     │
                               └───────────────────┘
```

P0 is not negotiable and not splittable: shipping durable cancel without the fence produces a system that reports success at stopping while still writing, which is worse than today because it invites the user to trust it.

## Choosing the success metrics

| Metric | Definition | Target | Instrument |
| --- | --- | --- | --- |
| Cancel honour time | p99 seconds from accepted cancel to last side effect | ≤ 5 s with notify, ≤ 65 s without | `cancel_to_stop_seconds` histogram |
| Cancel completeness | share of cancels with zero residue on verification | 100 % | purge verification step |
| Delete determinism | p99 delete latency at 10⁶ workspace rows / at 10³ | ratio ≤ 2 | benchmark suite |
| Fairness error | max deviation of consumed capacity from weighted share over 5 min | ≤ 10 % | fairness ledger |
| Starvation | count of tasks waiting beyond 20× median wait | 0 | per-tenant wait percentiles |
| Dead letter rate | tasks reaching `dead_letter` per 1000 | ≤ 2 | state counters |
| Status honesty | interface states with no stored counterpart | 0 | contract test, see Lens 8 |

## Accepting the risks

| Risk | Why it is real here | Mitigation |
| --- | --- | --- |
| Migration of live queues | tasks in flight during a state-enum change | add states additively, dual-read, never rename existing values |
| Fence rejects legitimate writes | a slow but valid ingest looks stale after an unrelated bump | bump the epoch only on destructive operations, never on retries |
| Keeping cancelled rows grows the table | today `cancel_and_delete_task` deletes them | `prune_terminal_tasks` already exists; give it a retention policy per state |
| Ledger becomes a hot row | one counter row per tenant and lane | see Lens 4 on contention and batching |
| Longer perceived cancel | "Stopping" is now honest and visible | Lens 8 defines the affordance so honesty reads as competence |

## Where to read next

Contract and event shapes are in Lens 2. The row-level model that makes these criteria testable is in Lens 3. The claim ranking that delivers the fairness criteria is in Lens 4. Operational targets behind the metrics table are in Lens 5. Model-level cancellation cost is in Lens 7. Wording of every status is in Lens 8.