# 008 — O(n) Expert Lens

**Spec:** SPEC-057  
**Key question:** What is the asymptotic cost of ingestion, and where do timeouts/fairness break?

---

## Scope

Complexity classes per phase, coupling costs, scalability bounds. Cross-ref SPEC-038 measurements. Out of scope: micro-optimizing Cypher strings.

---

## Cost model by phase

| Phase | Dominant cost | Class | Notes |
| ----- | ------------- | ----- | ----- |
| HTTP admit | bytes | O(B) | BYTEA insert; sync before 200 |
| Vision convert | pages × LLM | **O(P · L)** | Default-risk class |
| EdgeParse convert | pages CPU | **O(P)** | Correct for born-digital |
| Chunk | text size | O(T) ≈ O(P) for PDF strategy |
| Extract | chunks × LLM / concurrency | **O(C · L / k)** | k = max concurrent extractions |
| Gleaning | entities × passes | O(E · G) | Often disabled locally |
| Embed | chunks / batch | O(C / b) provider calls |
| Graph merge | entities + relationships × DB RT | **O(E + R)** unbounded RT | Contention under multi-tenant |
| Persist compensate | prior writes | O(C + E) deletes | On merge fail |

---

## Vision vs EdgeParse (ASCII)

```text
  Pages P = 600

  Vision:     T ≈ t0 + P × t_page_llm     →  hours possible
              Worker timeout 7200s        →  FAIL class timeout_phase_convert

  EdgeParse:  T ≈ c × P                   →  minutes
              + Extract O(C/k)            →  often dominates after convert

  Coupled task (TODAY):
  T_total = T_convert + T_extract + T_embed + T_merge
  One worker slot + one tenant permit held for T_total
```

---

## Fairness asymptotics

```text
  Let W = WORKER_THREADS, M = MAX_TASKS_PER_TENANT

  Without limiter: one tenant can occupy min(W, queue) slots → starvation O(W)
  With park:       active_per_tenant ≤ M; waiters sleep (not O(requeue storms))

  Local clamp M=1: serializes tenant; protects GPU; increases queue latency O(N_tenant_jobs)

  Coupling penalty: long PDF+KG job holds permit for T_convert+T_kg
  → effective fairness window ≈ T_total, not T_phase
```

**Mitigation class:** split phases (REQ-057-07) so convert release permit before KG, or checkpoint barrier with permit refresh.

---

## Scalability bottlenecks (ranked)

1. **O(P·L) Vision default** on large PDFs (SPEC-038)  
2. **Single-task lease** spanning convert+KG  
3. **Merge O(E+R) DB round-trips** under concurrent tenants  
4. **Embed batch limits** → permanent `embedding_limit` if unbounded fan-in  
5. **Checkpoint jsonb size** → slim omit embeddings → re-embed O(C) on resume  
6. **In-memory channel capacity** → admit backpressure vs silent drop risk under overload  

---

## Recommendations → REQ

| Change | Complexity effect | REQ |
| ------ | ----------------- | --- |
| EdgeParse admission for text-native | O(P·L) → O(P) | REQ-057-08, 11 (038) |
| Adaptive timeout f(P, backend, provider) | timeout matches class | REQ-057-08 |
| Split convert / ingest tasks | lease ≈ phase, not sum | REQ-057-07 |
| Chunked merge / native writes | reduce merge RT constant | REQ-057-12 |
| Bound checkpoint + slim policy | memory O(1) soft caps | REQ-057-14 |

**Out of scope:** Changing chunk strategy heuristics beyond pointing at LargeDocumentProfile SSOT.

Next: [009-postgres-relational-lens.md](./009-postgres-relational-lens.md)
