# Lens 8 — UX and UI Expert: Status Vocabulary, Stopping Affordance, Queue Transparency

> Parent: [SPEC-120 hub](https://app.notion.com/p/SPEC-120-First-Class-Task-System-for-EdgeQuake-Ingestion-Deletion-Reprocess-f2154512c0514e8e8d10cfbbc3f87c2b?pvs=21). Normative for what a person sees and can do. The durable states come from Lens 3, the payloads from Lens 2, and the timing promises from Lens 5. This lens adds no new states — it maps them.
> 

## Deriving the badge from the truth, once

Today `ingestion_status_mapper.rs` computes two presentation fields, `display_status` and `ui_phase` (`idle | running | stopping | terminal`), from the five durable statuses plus cancel intent. That mapper is the right idea in the wrong place: it is a second vocabulary, invented in the API, that the web interface then re-interprets. The fix is one table, generated from the durable state, shipped in the OpenAPI snapshot, and consumed verbatim.

| Durable state | `cancel_requested_at` | Badge | Tone | Stop button | Progress |
| --- | --- | --- | --- | --- | --- |
| `queued` | null | Queued | neutral | Cancel | none |
| `queued` | set | Cancelling… | warning | disabled | none |
| `held` | null | Waiting for capacity | neutral | Cancel | none |
| `leased` | null | Starting… | info | Cancel | indeterminate |
| `running` | null | *stage name* | info | Stop | determinate if total known |
| `running` | set | Stopping… | warning | disabled, shows countdown | frozen, dimmed |
| `cancelling` | set | Stopping… | warning | disabled | frozen, dimmed |
| `succeeded` | — | Ready | success | none | 100 % |
| `failed` | — | Failed, retrying | warning | Cancel | none |
| `cancelled` | — | Cancelled | neutral | none | none |
| `dead_letter` | — | Needs attention | danger | none | none |

Two consequences worth stating explicitly. "Failed, retrying" and "Needs attention" are different words because they demand different behaviour from the reader, and the first is not a call to action. And a state never silently reverts: once "Stopping…" is shown, the only successors are "Cancelled" or, in the case of a race the user lost, "Ready" with an explanation.

## Honouring the stop

```
┌───────────────────────────────────────────────────────────┐
│  contract-q3.pdf                                     [ Stop ]     │
│  ● Extracting text · page 214 of 900                             │
│  █████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  24 %              │
└───────────────────────────────────────────────────────────┘
       │ user presses Stop → optimistic switch within 100 ms
       ▼
┌───────────────────────────────────────────────────────────┐
│  contract-q3.pdf                                   Stopping…     │
│  ◐ Finishing the current page, then cleaning up · ~6 s            │
│  █████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  dimmed, frozen    │
└───────────────────────────────────────────────────────────┘
       │ countdown from expected_stop_by (Lens 2)
       │ if exceeded by 2×:
       ▼
│  ⚠ Still stopping. This can take up to two minutes if the worker
│    became unavailable.                        [ Details ]
       │
       ▼
┌────────────────────────────────────────────────────────────┐
│  contract-q3.pdf                       Cancelled  [ Start again ] │
│  Nothing was added to your knowledge base.                        │
└───────────────────────────────────────────────────────────┘
```

The closing sentence is the part users actually care about: cancellation makes a promise about data, and the interface should state the outcome rather than leaving it inferable. That promise is only true because of the retraction logic in Lens 7 and the fence in Lens 3.

### The interface state machine

```
                 ┌─────────┐
       enqueue   │ QUEUED  │◄─────────────────────────┐
      ────────►│ Queued  │                        retry │
                 └───┬─────┘                              │
          capacity │  │ no capacity                       │
                   │  ▼                                   │
                   │ ┌───────────────────┐            │
                   │ │ WAITING FOR CAPACITY │            │
                   │ │ + queue position     │            │
                   │ └─────────┬─────────┘            │
                   ▼          │                         │
               ┌────────────┐◄────┘                         │
               │ RUNNING     │  stage label + progress   │
               └──┬───────┬──┘                          │
          stop │       │ done                            │
               ▼       ▼                                 │
       ┌───────────┐  ┌─────────┐                       │
       │ STOPPING…  │  │ READY   │  terminal             │
       └────┬──────┘  └─────────┘                       │
            │ │ lost the race → READY + "finished before  │
            │ │ it could stop"                            │
            ▼                                             │
    ┌────────────┐      ┌──────────────────┐          │
    │ CANCELLED  │      │ NEEDS ATTENTION  │──────────┘
    └────────────┘      └──────────────────┘

The interface has six states to the durable nine: queued and held are shown
separately because the remedy differs, leased and running collapse into one,
failed-with-retries-left collapses into RUNNING (nothing is asked of the user),
and dead_letter surfaces as NEEDS ATTENTION.
```

## Deleting without lying

```
DELETE, WITH THE CANCEL-FIRST SEQUENCE MADE VISIBLE

[ Delete ]  →  ┌──────────────────────────────────────────┐
                │ Delete contract-q3.pdf?                  │
                │                                          │
                │ It is still being processed. We will stop │
                │ that first, then remove it everywhere.    │
                │                                          │
                │          [ Cancel ]  [ Delete ]           │
                └──────────────────────────────────────────┘

Then a single row, not four:
   Stopping processing…     →  Removing from search…  →  Deleted
   ●─────────────────────○──────────────────────────────○
   (the saga states of Lens 3, collapsed to what a person needs)

UNDO WINDOW
   Before the fence is raised, offer Undo for 5 seconds.
   After the fence, the operation is irreversible and Undo is not offered —
   never show an affordance the system will answer with 423 Locked.
```

The warning text is conditional on there being active work; when nothing is in flight, showing it teaches users to ignore the dialog. This is why the interface needs the active-task lookup from Lens 4 to be fast enough to run before rendering the dialog.

## Showing the queue as a queue

The most consequential copy change in this specification: `tenant_at_capacity` is currently surfaced as a rejection, and users read rejections as failures. It is not a failure — it is a queue, and queues are acceptable when they are legible.

```
┌────────────────────────────────────────────────────────────┐
│  Your queue                                          3 running     │
│                                                                    │
│  contract-q3.pdf        Extracting · 214 / 900                      │
│  minutes-jan.docx       Embedding · 12 / 40                         │
│  scan-batch-07.pdf      Converting · page 3                         │
│  ─────────────────────────────────────────────────────────  │
│  report-2024.pdf        Waiting · next in your queue                │
│  appendix-b.pdf         Waiting · 2nd                               │
│                                                                    │
│  Waiting because your other documents are still processing,         │
│  not because of a problem.                      [ Learn more ]      │
└────────────────────────────────────────────────────────────┘

Never show other tenants' positions or counts. "Next in your queue" is
honest and safe; "position 47 of 312" leaks a competitor's activity.
```

## Rendering uncertainty rather than inventing certainty

| Situation | Rendering |
| --- | --- |
| Total known, advancing | determinate bar, item counts |
| Total unknown (page count not yet parsed) | indeterminate shimmer plus the stage name |
| No progress for longer than expected, still alive | "Taking longer than usual" under the bar, bar unchanged |
| Retrying after a failure | "Retrying · attempt 2 of 3", progress resets visibly |
| Stopping | bar frozen and dimmed, never reset to zero |

A frozen bar during "Stopping…" is deliberate. Resetting it implies work was lost, animating it implies work continues, and freezing it says the truth: the work is being unwound.

## Recovering from the bad states

```
NEEDS ATTENTION card (dead_letter)

┌─────────────────────────────────────────────────────────┐
│  ⚠ scan-batch-07.pdf could not be processed                    │
│                                                                │
│  What happened : the text extraction service did not respond   │
│                  after 3 attempts.                             │
│  Your data     : nothing was added to your knowledge base.     │
│  What you can do: try again, or upload a text-based version.   │
│                                                                │
│  [ Try again ]  [ Remove ]           Reference: track-8f2a…    │
└─────────────────────────────────────────────────────────┘

Three lines, always in this order: what happened, what it means for your
data, what you can do. The reference is the track id, shown because support
conversations need it and hiding it costs a round trip.
```

## Making it usable for everyone

State changes are announced through a polite live region, so a screen-reader user hears "Stopping contract-q3.pdf" without losing their place. Focus stays on the stop control after it is pressed and the control becomes disabled with an accessible name of "Stopping, please wait" rather than disappearing, because a vanishing button destroys keyboard context. Every state is distinguishable without colour, using an icon and a text label, since "Failed" and "Ready" as red and green dots are indistinguishable to a significant share of users. Indeterminate animation respects `prefers-reduced-motion`, falling back to a static label. And bulk selection announces its scope before acting: "Stop 14 documents?" with an explicit count, never an unquantified "Stop all".

## Testing what the user experiences

| Scenario | Assertion |
| --- | --- |
| Press Stop on a running ingest | badge reads "Stopping…" within 200 ms without waiting for the server |
| Cancel confirmed by the server | badge reaches "Cancelled" and never returns to a running label |
| Task completes before the cancel lands | badge reads "Ready" with an explanatory line, no silent flip |
| Delete while running | one row shows stopping, then removing, then deleted |
| Reload the page mid-stop | state is restored from the server, not from local optimism |
| Tenant at capacity | queue panel explains the wait; no error styling appears |
| Dead letter | card shows the three lines and the track reference |

The existing end-to-end coverage in `spec057-cancel-status-ssot.spec.ts` already asserts the "Stopping…" badge, which makes it the natural home for these cases.

## Where to read next

The payload fields behind every label — `state`, `cancel_requested_at`, `cancellable_until`, `expected_stop_by` — are specified in Lens 2. The durable states are in Lens 3, the timing promises in Lens 5, and the progress semantics in Lens 7. The success metric for status honesty is in Lens 1.