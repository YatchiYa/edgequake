# Lens 4 — Postgres Expert: Claim SQL, Fair Queueing, Indexes, Vacuum

> Parent: [SPEC-120 hub](../SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md). Normative for physical schema, claim statements, and storage behaviour. The logical model it implements is in Lens 3.
> 

## Respecting what the branch already got right

Using Postgres as the queue is the correct decision for this product: the tasks, the documents, and the tenant metadata live in one database, so claiming and bookkeeping can be one transaction. The branch implements the canonical pattern — `claim_next_with_policy` with `FOR UPDATE SKIP LOCKED`, a lease token, `refresh_lease`, `release_claim`, monthly partitions via `ensure_month_partitions`, and terminal pruning via `prune_terminal_tasks`. This lens changes none of that; it removes the scans, adds the fairness ledger, and protects the hot table.

## Eliminating the JSON scans

The delete path is the worst query in the system today. `purge_persisted_tasks_for_document_except` lists with `Pagination { page: 1, page_size: 10_000 }` and then filters in Rust with `task_references_document`, checking `existing_document_id`, `document_id`, and `metadata.document_id`.

```sql
-- Promote the identifiers that the domain actually joins on.
ALTER TABLE tasks
  ADD COLUMN document_id UUID
    GENERATED ALWAYS AS (
      COALESCE(
        NULLIF(payload->>'existing_document_id',''),
        NULLIF(payload->>'document_id',''),
        NULLIF(payload#>>'{metadata,document_id}','')
      )::uuid
    ) STORED,
  ADD COLUMN pdf_id UUID
    GENERATED ALWAYS AS (NULLIF(payload->>'pdf_id','')::uuid) STORED;

-- One index turns an O(workspace) scan into an O(matching rows) lookup.
CREATE INDEX CONCURRENTLY tasks_document_active_idx
  ON tasks (document_id)
  WHERE state NOT IN ('succeeded','cancelled','dead_letter');

CREATE INDEX CONCURRENTLY tasks_pdf_active_idx
  ON tasks (pdf_id)
  WHERE state NOT IN ('succeeded','cancelled','dead_letter');
```

The COALESCE ordering encodes the same precedence the Rust helper applies, in one place, which is the DRY fix: the three-path probe stops being duplicated between `task_references_document` and every other call site that needs the same answer.

The dependent-cancel step of the deletion saga then becomes a single statement, replacing both the scan and the `for _ in 0..8` chain walk:

```sql
WITH pipeline AS (
  SELECT id FROM tasks
   WHERE document_id = $1
     AND state NOT IN ('succeeded','cancelled','dead_letter')
  UNION
  SELECT t.id FROM tasks t                     -- convert → insert children
    JOIN tasks p ON t.parent_task_id = p.id
   WHERE p.document_id = $1
     AND t.state NOT IN ('succeeded','cancelled','dead_letter')
)
UPDATE tasks SET state = 'cancelling',
                 cancel_requested_at = now(),
                 updated_at = now()
 WHERE id IN (SELECT id FROM pipeline)
RETURNING id, track_id;
```

## Claiming with quota and deficit ranking

The present policy compares an active count against a static cap. The target adds *share* without giving up `SKIP LOCKED`. Ranking happens inside the claim statement so no advisory locks and no second round trip are needed.

```sql
WITH lane AS (                                   -- durable, cross-replica truth
  SELECT tenant_id, fairness_class,
         COUNT(*) AS active
    FROM tasks
   WHERE state IN ('leased','running')
   GROUP BY 1,2
),
quota AS (
  SELECT w.tenant_id, w.fairness_class, w.weight,
         w.max_concurrent,
         COALESCE(l.active,0) AS active,
         COALESCE(v.vruntime,0) AS vruntime      -- deficit round robin clock
    FROM tenant_lane_quota w
    LEFT JOIN lane l USING (tenant_id, fairness_class)
    LEFT JOIN tenant_vruntime v USING (tenant_id, fairness_class)
),
candidate AS (
  SELECT t.id,
         ROW_NUMBER() OVER (
           ORDER BY q.vruntime / GREATEST(q.weight,1) ASC,  -- fair share first
                    t.priority DESC,
                    t.created_at ASC                        -- then arrival
         ) AS rn
    FROM tasks t
    JOIN quota q
      ON q.tenant_id = t.tenant_id
     AND q.fairness_class = t.fairness_class
   WHERE t.state = 'queued'
     AND t.available_at <= now()
     AND (t.hold_until IS NULL OR t.hold_until <= now())
     AND t.cancel_requested_at IS NULL
     AND q.active < q.max_concurrent            -- quota gate
   ORDER BY 2
   LIMIT 20                                     -- small window, cheap sort
)
UPDATE tasks t
   SET state = 'leased', updated_at = now()
  FROM (
    SELECT id FROM candidate WHERE rn = 1
     FOR UPDATE SKIP LOCKED
  ) picked
 WHERE t.id = picked.id
RETURNING t.*;
```

After the attempt finishes, the ledger is charged with what was actually consumed rather than with one unit:

```sql
INSERT INTO tenant_vruntime (tenant_id, fairness_class, vruntime, updated_at)
VALUES ($1, $2, $3, now())
ON CONFLICT (tenant_id, fairness_class) DO UPDATE
  SET vruntime = GREATEST(tenant_vruntime.vruntime, $4_min_virtual_clock) + $3,
      updated_at = now();
```

The `GREATEST` against a global minimum is the standard virtual-clock reset: a tenant returning after an idle period starts level with the current frontier instead of accumulating an unbounded credit that would let it monopolise the queue on return.

```
DEFICIT ROUND ROBIN, TWO TENANTS, WEIGHT 1:1

  vruntime
     │
  60 ┤                                   ██ X
     │                          ██ X     ██ Y
  40 ┤                 ██ X     ██ Y     ██
     │        ██ X     ██ Y     ██
  20 ┤ ██ X  ██ Y     ██
     │ ██ Y  ██
   0 └────────────────────────────────────► time

  X has 1000 queued tasks, Y has 1. Y is picked as soon as its vruntime is the
  lowest, i.e. within one claim cycle, because ranking is by consumed capacity,
  not by backlog depth or arrival order.
```

### Contention control on the ledger

One row per `(tenant, lane)` is a hot row when a tenant runs hundreds of tasks per minute. Three mitigations, in order of preference: charge on attempt completion rather than on progress; batch charges from each worker every few seconds instead of per chunk; if a single tenant still saturates one row, shard it into `N` sub-rows keyed by `hashtext(worker_id) % N` and sum on read. Do not use advisory locks — they serialise the claim path that `SKIP LOCKED` exists to keep parallel.

## Indexing the hot path

```sql
-- The claim predicate, as a partial covering index.
CREATE INDEX CONCURRENTLY tasks_claim_idx
  ON tasks (fairness_class, tenant_id, available_at, created_at)
  INCLUDE (id, priority)
  WHERE state = 'queued' AND cancel_requested_at IS NULL;

-- Cancellation sweep and drain wait.
CREATE INDEX CONCURRENTLY tasks_cancelling_idx
  ON tasks (cancel_requested_at)
  WHERE state = 'cancelling';

-- Lease expiry reaper (orphan_task_recovery.rs).
CREATE INDEX CONCURRENTLY attempts_expiry_idx
  ON attempts (lease_expires_at)
  WHERE finished_at IS NULL;

-- Time-range reporting over a partitioned, append-mostly table.
CREATE INDEX CONCURRENTLY tasks_created_brin
  ON tasks USING brin (created_at) WITH (pages_per_range = 32);
```

The partial predicates matter more than the column lists. A queue table's index must not carry terminal rows: with a partial index the index size tracks the *backlog*, not the *history*, so a workspace with a million completed tasks pays nothing on the claim path. This is what makes hub invariant INV-7 achievable.

## Protecting the table from its own writes

```
WRITE AMPLIFICATION TODAY

  heartbeat every 60 s × N running tasks → UPDATE tasks SET lease_expires_at
  progress updates     × many per task   → UPDATE tasks SET progress
                                             │
                                             ▼
                   every update writes a new row version on the SAME table
                   the claim query scans, so the claim index bloats and
                   autovacuum races the workers

TARGET

  tasks      ← low-churn: state transitions only
  attempts   ← lease and heartbeat churn lives here
  task_events← append-only progress, never updated

  The scheduler's index no longer competes with heartbeat traffic.
```

Complementary settings on the churn tables:

```sql
ALTER TABLE attempts SET (
  fillfactor = 70,                       -- room for HOT updates in-page
  autovacuum_vacuum_scale_factor = 0.01, -- vacuum early, this table is hot
  autovacuum_vacuum_cost_delay = 0
);
ALTER TABLE tasks SET (fillfactor = 90);
```

Avoid `HOT` breakage by never indexing a column that the heartbeat updates. That rule is why `lease_expires_at` must not live on `tasks` next to the claim index.

## Partitioning and retention

```
tasks  PARTITION BY RANGE (created_at)      ← already present via ensure_month_partitions
 ├─ tasks_2026_05   detached and dropped when past retention
 ├─ tasks_2026_06
 ├─ tasks_2026_07   ◄ current, hot, small
 └─ tasks_default   ◄ keep, but alert if it ever receives rows

Retention by state, enforced by prune_terminal_tasks:
   succeeded    30 days      (analytics only)
   cancelled    90 days      (audit: who stopped what, and when)
   dead_letter  365 days     (incident forensics)
   non-terminal never pruned (a pruned live task is a lost operation)

DROP the partition; never DELETE the rows. A monthly DROP is instant and
produces no bloat, whereas a bulk DELETE produces exactly the dead tuples
that degrade the claim index.
```

## Waking replicas without polling

```
ENQUEUE                                       CLAIM LOOP (every replica)
  INSERT INTO tasks …                           LISTEN task_ready;
  pg_notify('task_ready', tenant_id)            LISTEN task_cancel;
                                                loop {
CANCEL                                            select! {
  UPDATE tasks SET cancel_requested_at…            notif = conn.recv() => claim(),
  pg_notify('task_cancel', track_id)               _ = sleep(2s)      => claim(),
                                                  }
                                                }

NOTIFY is a latency optimisation only. The periodic tick is the correctness
guarantee, because NOTIFY is not durable and is lost across a reconnect.
This is the same discipline the branch already applies to its wake channel.
```

## Avoiding the classic queue anti-patterns

| Anti-pattern | Present risk | Rule |
| --- | --- | --- |
| Long-lived transaction around the work | worker holds a snapshot for minutes, blocking vacuum | claim commits immediately; work runs outside any transaction — already correct on this branch |
| `OFFSET` pagination on a large table | `page_size: 10_000` in the delete path | keyset only; `Pagination::has_keyset_cursor` already exists, use it |
| `SELECT FOR UPDATE` without `SKIP LOCKED` | thundering herd across replicas | keep `SKIP LOCKED` |
| `count(*)` for queue depth | sequential scan per metrics scrape | maintain counters, or accept an estimate from the partial index |
| Indexing every column | write amplification on a hot table | four partial indexes, nothing more |
| Storing joinable identity in JSONB | the delete scan | generated columns, as above |

## Where to read next

The logical model these statements implement is in Lens 3. Heartbeat interval and lease TTL choices that drive the drain bound are in Lens 5. The Rust ports that own these statements are in Lens 6. Cost estimation that feeds `vruntime` is in Lens 7. The fairness explanation shown to users is in Lens 8.