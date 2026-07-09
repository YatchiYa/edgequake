# SPEC-045 — First Principles (Ingestion × Migration)

**Cross-ref:** [003-code-is-law](./003-code-is-law.md) · [006-bulletproof-migration-design.md](./006-bulletproof-migration-design.md)

---

## Invariant: successful ingestion

A document ingestion succeeds **if and only if** all of the following hold at commit time:

```
I1  Content extracted     — chunks.length > 0 OR explicit empty-content policy
I2  Embeddings written    — vector dim == table dim; batch limits respected
I3  Graph merged          — entities + relationships upserted; stats.errors == 0
I4  Metadata consistent   — KV status terminal; wsdoc index present; PG row synced
I5  Task terminal         — task status completed; no orphan processing lock
```

Violation of **any** invariant → document `failed` or stuck non-terminal state.

---

## Pipeline as a saga

Ingestion is a **distributed saga** across four stores:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  KV store   │    │ Vector store│    │ Graph (AGE) │    │ PG documents│
│  metadata   │    │  chunks     │    │  entities   │    │  read model │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │                  │
       └──────────────────┴──────────────────┴──────────────────┘
                              │
                    ingestion_persister
                              │
                    on failure → compensate_merge_failure
```

**First principle:** Merge failure must **compensate** (delete orphan nodes/edges + chunk vectors) or operators get duplicate entities on reprocess.

**SPEC-044 lesson:** Compensation paths using `cypher_execute_bound` must use AGE's bare `$1` third argument — not inline literals.

---

## Migration bootstrap interaction

Bootstrap runs **before** the API accepts traffic (readiness) and **before** workers process tasks:

```
startup sequence (main.rs)
─────────────────────────
1. run_postgres_migrations()     ← sqlx + reconcile m038..m081
2. is_ready_for_traffic()        ← /ready gate (M038, M042)
3. recover_orphaned_tasks()      ← processing → pending
4. recover_orphaned_documents()  ← non-terminal → pending/failed
5. requeue_pending_tasks()
6. start workers
```

**First principle:** Orphan recovery **must** precede workers — otherwise race with new uploads.

---

## What migrations affect ingestion?

### Traffic-blocking (readiness)

| Migration | Condition | Operator action |
| --------- | --------- | --------------- |
| M038 | `source_ids` GIN indexes missing on AGE graph | `apply_038.sh --concurrent` |
| M042 | pgvector < 0.8 (no iterative scan) | Rebuild postgres image / `SQL_042_APPLY` |

### Ingestion-degrading (non-blocking)

| Migration | Condition | Symptom |
| --------- | --------- | ------- |
| M038 deferred | Large graph, indexes pending | Slow merge, timeouts |
| M046 | Tenant isolation indexes missing | Slow scoped queries |
| M047 | wsdoc index incomplete | List/search miss docs |
| M040 | CQRS entity backfill lagging | Entity count mismatch in UI |
| M080 | halfvec conversion partial | Vector insert errors |

### Not ingestion-related

M048–M065 (auth PG SSOT) — affect login, not document pipeline.

---

## Failure classification first principles

Every terminal failure should answer three operator questions:

1. **What broke?** → `failure_class` (stable enum key)
2. **What should I do?** → `recommended_action` (verb phrase)
3. **Can I retry safely?** → idempotency of reprocess path

Current `IngestionFailureClass` covers timeout, embedding, provider — **missing `graph_merge`**.

Merge failures today:

```
error_message: "1 knowledge-graph merge error(s) during persist"
failure_class: "unknown"          ← BUG
recommended_action: "retry"       ← WRONG (needs graph cleanup first)
```

Correct mapping:

```
failure_class: "graph_merge"
recommended_action: "reprocess_full"  ← triggers cleanup_document_graph_data
```

---

## Reprocess idempotency

Reprocess is safe **only when**:

1. `cleanup_document_graph_data` runs first (graph + vectors for doc)
2. `purge_persisted_tasks_for_document` clears in-flight task
3. Checkpoint invalidated if provider/content-hash changed

**Edge case:** Reprocess without cleanup → duplicate entities (OODA-08).

---

## Dual-store read model

Documents exist in **KV** (pipeline SSOT) and **PostgreSQL** `documents` table (list API).

Merge rule in `document_read_model.rs`:

- Status: normalize `completed` (KV) ↔ `indexed` (PG)
- Counts: `max(kv, pg)` for entity/chunk counts
- Cost stats: read from `metadata` JSONB (not M041 columns — avoids partial migration breakage)

**First principle:** List API must not depend on migration-041 columns being present.

---

## Provider dimension alignment

```
embedding_dim(provider) == vector_table_dim(workspace)
```

Mismatch triggers:

- Insert failure at persist (hard error)
- Query `dimension_mismatch` auto-retry (OODA-225)

After migration + provider switch, run `rebuild embeddings` or verify `EDGEQUAKE_VECTOR_STORAGE` + M080 state.

---

## Decision: auto-migration vs operator script

| Work type | Auto at bootstrap | Operator script |
| --------- | ----------------- | --------------- |
| Idempotent DDL < 30s | ✅ reconcile every boot | — |
| Index on graph > 100k nodes | Defer + degrade `/ready` | CONCURRENTLY script |
| Data backfill > 1M rows | Background tokio::spawn | Monitor progress |
| Failed doc recovery | Startup orphan + API endpoints | Manual reprocess |

**First principle:** Never block startup on unbounded backfill; always surface progress in `/health`.
