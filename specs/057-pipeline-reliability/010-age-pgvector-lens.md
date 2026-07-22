# 010 — AGE + pgvector Lens

**Spec:** SPEC-057  
**Key question:** How do graph and vector stores behave under ingest failure, cancel, and multi-tenant load?

---

## Scope

Persist order, saga compensation, HNSW/native writes, merge contention. Cross-ref SPEC-042 / SPEC-045 / SPEC-046-10. Out of scope: query-time graph algorithms.

---

## Persist order + compensation (ASCII)

```text
  DefaultIngestionPersister
           │
           ▼
     (1) Chunk KV upsert ──────────────────────────┐
           │                                       │
           ▼                                       │
     (2) Chunk vector upsert (pgvector) ───────────┤
           │                                       │
           ▼                                       │
     (3) KnowledgeGraphMerger (AGE)                │
           │                                       │
      success ──► community refresh / metrics      │
           │                                       │
      failure ──► compensate_merge_failure ────────┘
                    delete orphan vectors / KV
                    (+ partial graph cleanup)
```

**Law:** Not 2PC. Crash between (2) and compensate ⇒ orphan window (CAUSE-057-07).

---

## Findings

### Strengths

- Explicit saga with `compensate_merge_failure` in persister.  
- Native AGE upserts (`EDGEQUAKE_NATIVE_GRAPH_WRITES`) reduce Cypher MERGE cost.  
- Vector upserts idempotent by id.  
- `graph_merge` permanent class avoids useless retries (SPEC-045).

### Risks

| Risk | Mechanism | Scale effect |
| ---- | --------- | ------------ |
| Orphan window | Crash mid-saga | Queryable chunks without graph |
| Merge contention | Concurrent tenants writing AGE | Latency spikes / timeouts |
| HNSW build/query tuning | ef_construction / iterative_scan | Ingest vs query tradeoff |
| Cancel mid-persist | Cooperative cancel between steps | May leave partial writes until compensate/reprocess |
| Compensation failure | Delete path errors | Silent dual-store drift |

---

## Cancel semantics in stores

Cancel before persist: ideal — little/no store mutation.  
Cancel during merge: token checks should abort; compensation may still run on failure paths — must be **idempotent** (REQ-057-11).  
Cancelled must not leave doc as query-complete.

---

## Multi-tenant contention model

```text
  Tenants T, merge cost M(E,R), concurrency ≈ min(W, Σ permits)

  AGE write amplification ≈ T_active × M
  pgvector upsert amplification ≈ T_active × C_chunks

  Fairness park reduces T_active per tenant but global W still fans into one DB
  ⇒ need store budgets (REQ-057-12), not only tenant task caps
```

---

## Recommendations → REQ

| Change | REQ |
| ------ | --- |
| Idempotent compensate + metric/DLQ on compensate fail | REQ-057-11 |
| Publish store contention SLOs on queue-metrics / ready | REQ-057-12 |
| Prefer native graph writes in prod ingest path | (env default on) |
| Reprocess cleans graph artifacts before requeue (SPEC-045 REQ) | REQ-057-13 lineage |
| Bound merge batching (SPEC-047 chunked merge) | REQ-057-12 |

**Out of scope:** Migrating off AGE; halfvec dimension strategy (SPEC-042).

Next: [011-ai-engineer-lens.md](./011-ai-engineer-lens.md)
