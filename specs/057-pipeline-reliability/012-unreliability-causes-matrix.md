# 012 — Unreliability Causes Matrix

**Spec:** SPEC-057  
**Method:** Each cause has symptom → 5-Why chain → root layer → **roadblock** → **mitigation** → priority → REQ → proof.

---

## Register

| ID | Symptom | Why | Root layer | Roadblock | Mitigation | P | REQ | Proof |
| -- | ------- | --- | ---------- | --------- | ---------- | - | --- | ----- |
| CAUSE-057-01 | Work vanishes / never wakes after restart | [Chain A](./001-five-whys.md) | Delivery | `ChannelTaskQueue` is ephemeral; Pending rows not claimed | Postgres claim (`SKIP LOCKED`) + channel/NOTIFY as wake only | P1 | REQ-057-01 | claim unit + restart e2e |
| CAUSE-057-02 | Cancel ignored after restart / race | [Chain B](./001-five-whys.md) | Cancel | Intents in `CancellationRegistry` memory only | Treat DB `Cancelled` as durable intent; optional intent column; always re-read status on dequeue | P1 | REQ-057-02, 06, 15 | restart after cancel contract |
| CAUSE-057-03 | Cancel shows Failed on PDF | [Chain B](./001-five-whys.md) | Status | ~~`PdfProcessingStatus` lacks `Cancelled`~~ **FIXED P0** | Enum + mig 087 + cancel writers | P0 | REQ-057-03, 05, 15 | `e2e_pdf_cancel_sets_pdf_status_cancelled_not_failed` |
| CAUSE-057-04 | 2h timeout; long tenant lease | [Chain D](./001-five-whys.md) | Task model | PDF convert + `process_text_insert` inline in one task | Split Convert/Ingest tasks or hard checkpoint barrier + permit refresh | P2 | REQ-057-07 | timing/lease tests |
| CAUSE-057-05 | Orphan `processing` after crash | [Chain A](./001-five-whys.md) | Recovery | Auto-resume off; reconcile capped; no lease TTL | Lease expiry reaper + Interrupted UX → Reprocess; keep default auto-resume off | P1 | REQ-057-01, 05 | orphan reconcile proof |
| CAUSE-057-06 | Local thrash or false park | [Chain C](./001-five-whys.md) | Fairness | Clamp uses configured `EDGEQUAKE_LLM_PROVIDER`, not runtime extract | Key clamp off runtime provider used for extract/embed | P2 | REQ-057-09 | hybrid-provider test |
| CAUSE-057-07 | Chunks/vectors without graph | [Chain E](./001-five-whys.md) | Saga | Compensate-not-2PC crash window | Idempotent compensate; startup orphan scan; DLQ/metric on compensate fail | P3 | REQ-057-11, 12 | compensation e2e (core tests) |
| CAUSE-057-08 | Resume re-embeds / huge checkpoint | AI/cost | Checkpoint | Slim omits embeddings; jsonb soft size pressure | Bound checkpoint; metric re-embed; keep slim default | P2 | REQ-057-14 | SPEC-047 slim contracts |
| CAUSE-057-09 | Multi-replica double-work / miss | Scale-out | Delivery | Bridged/NotifyOnly exist; ops default single-process channel | Production path: Bridged/NotifyOnly + SQL claim | P3 | REQ-057-10 | `contract_spec026_task_delivery` |
| CAUSE-057-10 | UI/API status disagree | [Chain B](./001-five-whys.md) | Status | Task / doc KV / PDF / unified / core enums diverge | `IngestionStatusMapper` SSOT DTO | P0 | REQ-057-04 | mapper unit + UI badge |
| CAUSE-057-11 | Large PDF Vision timeout/bill | [Chain D](./001-five-whys.md) | Asymptotics | Vision O(P·L) vs EdgeParse O(P) | `LargeDocumentProfile` + admission routing (SPEC-038) | P2 | REQ-057-08 | SPEC-038 repro |
| CAUSE-057-12 | Wasteful retries / unknown fails | [Chain F](./001-five-whys.md) | Taxonomy | Novel errors → `Unknown` → retry | Extend `classify_ingestion_failure`; enforce permanent set | P0 | REQ-057-13 | SPEC-045 + classifier tests |

---

## Roadblock ↔ mitigation detail

### CAUSE-057-01 — Non-durable delivery

```text
  Roadblock:  mpsc is the wake AND the work copy
  Mitigation: Row is work copy; claim is wake; channel optional accelerator
  DoD:        Kill -9 mid-Pending → after restart claim runs task without manual SQL
```

### CAUSE-057-02 — Process-local cancel intent

```text
  Roadblock:  cancel_intents HashSet dies with process
  Mitigation: Worker dequeue: if task.status==Cancelled OR intent → drop
              Persist Cancelled before returning 200 from cancel API (already mostly true)
  DoD:        Cancel Pending → restart → task never processes
```

### CAUSE-057-03 — PDF status missing Cancelled

```text
  Roadblock:  enum { Pending, Processing, Completed, Failed }
  Mitigation: Add Cancelled; stop mapping user cancel → Failed in task_impl/pdf paths
  DoD:        Cancel PDF job → pdf row cancelled AND task Cancelled AND doc KV cancelled
```

### CAUSE-057-04 — PDF+KG coupling

```text
  Roadblock:  One TaskType::PdfProcessing does convert+KG
  Mitigation: PdfConvert task → checkpoint markdown → TextInsert/Ingest task
              OR same task with phase barrier releasing tenant permit between phases
  DoD:        Convert complete survives worker timeout during extract independently
```

### CAUSE-057-05 — Orphan processing

```text
  Roadblock:  Processing without lease; auto-resume off
  Mitigation: lease_until + reaper → Failed/Interrupted; UI Reprocess
  DoD:        Stale Processing (> lease) never blocks forever
```

### CAUSE-057-06 — Fairness clamp mismatch

```text
  Roadblock:  Env provider drives clamp
  Mitigation: Resolve provider from runtime Workspace/EdgeQuake config used by extract
  DoD:        OpenAI extract + Ollama env does not incorrectly clamp to 1 (unless extract local)
```

### CAUSE-057-07 — Saga window

```text
  Roadblock:  Multi-store writes without distributed TX
  Mitigation: Idempotent compensate; periodic orphan janitor; alert on compensate Err
  DoD:        Inject merge fail → no orphan vectors; inject compensate fail → metric fires
```

### CAUSE-057-08 — Checkpoint pressure

```text
  Roadblock:  Large ProcessingResult jsonb; slim needs re-embed
  Mitigation: Keep slim; cap size; surface "re-embedding" stage on resume
  DoD:        Resume never OOMs; UI shows re-embed stage
```

### CAUSE-057-09 — Horizontal incomplete

```text
  Roadblock:  Multi-instance without SQL claim races
  Mitigation: SKIP LOCKED claim required when replicas > 1; Bridged wake
  DoD:        Two API replicas do not double-process same track_id
```

### CAUSE-057-10 — Status fragmentation

```text
  Roadblock:  Multiple enums / string stages
  Mitigation: Single mapper module → API DTO; UI consumes only DTO
  DoD:        One fixture matrix covers all badge outputs
```

### CAUSE-057-11 — Vision asymptotic class

```text
  Roadblock:  Wrong backend for born-digital large PDFs
  Mitigation: Text-layer probe → EdgeParse; adaptive timeout from profile
  DoD:        SPEC-038 repro completes under timeout on EdgeParse path
```

### CAUSE-057-12 — Taxonomy gaps

```text
  Roadblock:  String match classifier incomplete
  Mitigation: Add classes as errors appear; never retry permanent
  DoD:        Golden error strings → expected class in unit tests
```

---

## Priority heatmap

```text
  P0  CAUSE-03, 10, 12          Controllability & truth
  P1  CAUSE-01, 02, 05          Restart durability
  P2  CAUSE-04, 06, 08, 11      Scale of a single job / provider
  P3  CAUSE-07, 09              Store saga + multi-instance
```

Next: [013-cross-reference-matrix.md](./013-cross-reference-matrix.md)
