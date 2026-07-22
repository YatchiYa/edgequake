# 001 — Five Whys (Ingestion Unreliability)

**Spec:** SPEC-057  
**Method:** User-visible symptom → systemic cause → `CAUSE-057-xx` in [012-unreliability-causes-matrix.md](./012-unreliability-causes-matrix.md)

---

## Chain A — Lost / stuck work after restart

| # | Why | Answer |
| - | --- | ------ |
| 1 | Why do documents stay `processing` or never resume after `make stop` / deploy? | Workers die mid-task; in-flight channel messages vanish. |
| 2 | Why don’t pending tasks automatically continue? | Hot-path delivery is `ChannelTaskQueue` (memory), not a DB claim loop. |
| 3 | Why isn’t the durable `tasks` table enough? | Rows persist, but nothing claims them unless `EDGEQUAKE_STARTUP_AUTO_RESUME=1`. |
| 4 | Why is auto-resume off by default? | SPEC-054: auto-hydrate can spend LLM quota and surprise operators. |
| 5 | Why is the system still unreliable with that policy? | Default path has **no wake + no explicit “interrupted → needs Reprocess” UX guarantee** for all surfaces; orphans require reconcile caps. |

**Systemic cause:** Ephemeral delivery + opt-in resume → [CAUSE-057-01](./012-unreliability-causes-matrix.md), [CAUSE-057-05](./012-unreliability-causes-matrix.md)

---

## Chain B — Cancel feels ignored / status inconsistency

| # | Why | Answer |
| - | --- | ------ |
| 1 | Why does Cancel not instantly stop progress? | Cancel is **cooperative** — waits for current `.await` / HTTP round-trip. |
| 2 | Why can UI still show Failed for a user cancel? | `PdfProcessingStatus` has only Pending/Processing/Completed/**Failed** — no Cancelled. |
| 3 | Why do task and PDF disagree? | Task row → `Cancelled` via `apply_task_row_cancel`; PDF path often maps cancel/abort to `Failed`. |
| 4 | Why isn’t there one mapper? | Task / doc KV / PDF / unified stage evolved as separate enums (stage_bridge deferred collapse). |
| 5 | Why does restart make cancel worse? | Cancel **intents** live in process memory; after restart only DB `Cancelled` remains (correct if written, invisible if race). |

**Systemic cause:** Status fragmentation + process-local intents → [CAUSE-057-02](./012-unreliability-causes-matrix.md), [CAUSE-057-03](./012-unreliability-causes-matrix.md), [CAUSE-057-10](./012-unreliability-causes-matrix.md)

---

## Chain C — Tenant starvation / local LLM thrash

| # | Why | Answer |
| - | --- | ------ |
| 1 | Why can one tenant block others? | Workers pull ready work; without a cap, one tenant can fill the pool. |
| 2 | Why was fairness added? | `TenantConcurrencyLimiter` parks excess tasks instead of 500ms requeue storms. |
| 3 | Why do local Ollama setups still thrash? | Local providers clamp to 1 concurrent task/tenant (good) but env LLM may ≠ extraction model. |
| 4 | Why does mismatch matter? | Clamp uses `EDGEQUAKE_LLM_PROVIDER` config; hybrid/runtime switch can under/over-limit. |
| 5 | Why does that feel “unreliable”? | Queue appears stuck (`tenant_park_waiters` > 0) while another tenant’s work could run — or local GPU is overwhelmed when clamp is lifted. |

**Systemic cause:** Fairness clamp keyed on configured env → [CAUSE-057-06](./012-unreliability-causes-matrix.md)

---

## Chain D — Long PDF+KG timeout / single worker slot

| # | Why | Answer |
| - | --- | ------ |
| 1 | Why do large PDFs hit Failed at ~2h? | `TASK_PROCESSING_TIMEOUT_SECS` default 7200 kills the whole task. |
| 2 | Why is the whole job one timeout? | `process_pdf_processing` converts then **inline** calls `process_text_insert`. |
| 3 | Why wasn’t convert separated from KG? | Historical single `PdfProcessing` task type; simpler admit path. |
| 4 | Why is Vision so expensive? | Vision is O(pages × LLM); EdgeParse is O(pages) CPU (SPEC-038). |
| 5 | Why does this hurt fairness? | One long task holds a worker + tenant permit for convert+extract+embed+merge. |

**Systemic cause:** Task coupling + asymptotic class → [CAUSE-057-04](./012-unreliability-causes-matrix.md), [CAUSE-057-11](./012-unreliability-causes-matrix.md)

---

## Chain E — Graph merge / vector–KV partial state

| # | Why | Answer |
| - | --- | ------ |
| 1 | Why can query see chunks without a complete graph? | Persist order: KV → vectors → AGE merge. |
| 2 | Why isn’t that transactional? | Cross-store writes cannot be one Postgres TX across all adapters; saga compensates on merge failure. |
| 3 | Why do orphans still appear? | Compensate-not-2PC: crash between vector upsert and compensate leaves a window. |
| 4 | Why isn’t compensation always visible? | Failed compensate historically silent; operators see `graph_merge` / unknown without DLQ. |
| 5 | Why does load amplify this? | Multi-tenant fan-out increases merge contention and crash probability mid-saga. |

**Systemic cause:** Saga window + contention → [CAUSE-057-07](./012-unreliability-causes-matrix.md), [CAUSE-057-12](./012-unreliability-causes-matrix.md)

---

## Chain F — Permanent failure burning retries

| # | Why | Answer |
| - | --- | ------ |
| 1 | Why do some docs retry 3× then fail the same way? | Worker retries retriable errors with backoff. |
| 2 | Why should some never retry? | Embedding 400, graph merge, circuit breaker, cancelled are permanent (SPEC-045). |
| 3 | Why do unknowns still burn budget? | `classify_ingestion_failure` string-matches; novel errors → `Unknown` → retry. |
| 4 | Why does that waste money/time? | LLM/embed calls re-run; tenant slot occupied. |
| 5 | Why isn’t classification complete? | Taxonomy is living SSOT; new error shapes need explicit classes. |

**Systemic cause:** Incomplete classification → [CAUSE-057-12](./012-unreliability-causes-matrix.md) (taxonomy adjacency SPEC-045)

---

## Summary map

```text
  User symptom              Systemic cause                 CAUSE
  ─────────────────────     ──────────────────────────     ──────────
  Stuck after restart  ──►  Channel + auto-resume off  ──► 01, 05
  Cancel inconsistency ──►  Status drift + local intent──► 02, 03, 10
  Tenant thrash/park   ──►  Env-keyed fairness clamp   ──► 06
  2h timeout / long job──►  PDF+KG coupling + Vision   ──► 04, 11
  Partial graph/KV     ──►  Compensate-not-2PC window  ──► 07
  Resume cost / OOM    ──►  Slim checkpoint re-embed   ──► 08
  Multi-replica races  ──►  Channel default, no claim  ──► 09
  Wasteful retries     ──►  Unknown failure_class      ──► 12
```
Next: [002-first-principles.md](./002-first-principles.md)
