# 011 — AI Engineer Lens

**Spec:** SPEC-057  
**Key question:** How do LLM/embed providers, retries, and checkpoints interact with reliability?

---

## Scope

Provider selection, concurrency clamps, chunk extract/embed retry, gleaning cost, failure_class, checkpoints. Out of scope: prompt quality tuning.

---

## Provider reliability surface

| Concern | Mechanism | File / knobs |
| ------- | --------- | ------------ |
| Local overload | Clamp workers + tenant concurrency to 1 | `resolve_worker_pool_limits`, `MAX_TASKS_PER_TENANT` |
| Hybrid mismatch | Env LLM ≠ extract/embed provider | `.env.example` warning |
| Chunk extract retry | Per-chunk retries + overload backoff | `pipeline/extraction.rs`, `EDGEQUAKE_CHUNK_*` |
| Embed retry | Transient backoff; 400 permanent | `helpers/embeddings.rs`, taxonomy |
| Circuit breaker | Task-level after repeated timeouts | `edgequake-tasks` task types |
| Vision vs EdgeParse | Backend choice dominates wall time | SPEC-038, `EDGEQUAKE_PDF_PARSER_BACKEND` |
| Gleaning | Extra LLM passes | Often off locally (`EDGEQUAKE_LOCAL_ENABLE_GLEANING`) |

---

## Checkpoint / resume (AI cost)

```text
  Extract complete ──► save ProcessingResult checkpoint
                       (slim: strip embeddings — SPEC-047 P5)
                              │
           resume ────────────┤
                              ▼
                    ensure_embeddings() ──► re-embed O(C)
                              │
                              ▼
                         persist / merge

  Durable extraction snapshot survives success for MergeOnly / soft reprocess
```

**Reliability win:** avoid re-paying extract LLM.  
**Cost caveat:** slim resume re-embeds; bound checkpoint size (REQ-057-14).

---

## Failure taxonomy (AI-facing)

From `IngestionFailureClass` (SPEC-045 SSOT):

| Class | AI meaning | Action |
| ----- | ---------- | ------ |
| `timeout_phase_convert` | Vision/convert too slow | EdgeParse / split |
| `timeout_phase_extract` | Chunk LLM slow | faster model / lower concurrency |
| `embedding_limit` | Provider 400 batch/token | shrink batches (permanent) |
| `provider_unavailable` | Ollama/API down | fix provider / reduce concurrency |
| `circuit_breaker` | Repeated timeouts | stop burning; reprocess smarter |
| `cancelled` | User stop | none |
| `graph_merge` | Not LLM — store | reprocess_full |
| `unknown` | Unclassified | improve classifier |

---

## Findings

### Strengths

- Layered retries (task / chunk / embed) with permanent short-circuit.  
- Local concurrency clamps protect small GPUs.  
- Cancel tokens threaded into vision/extract/embed awaits.  
- Checkpoints + snapshots reduce duplicate LLM spend on resume paths.

### Risks

1. Fairness clamp from **configured** LLM env, not runtime extract model (CAUSE-057-06).  
2. Unknown errors still burn retry + LLM $.  
3. Coupled PDF+KG means extract never starts if convert times out — and vice versa holds GPU slots.  
4. Auto-resume=1 can surprise-spend quota (SPEC-054).  

---

## Recommendations → REQ

| Change | REQ |
| ------ | --- |
| Clamp/fairness from runtime extract provider | REQ-057-09 |
| Expand classifier for new provider error shapes | REQ-057-13 |
| LargeDocumentProfile drives concurrency + timeout | REQ-057-08 |
| Keep slim checkpoints; metric re-embed cost | REQ-057-14 |
| Never auto-retry `cancelled` / permanent classes | REQ-057-06 |

**Out of scope:** New model evaluations; switching default cloud model IDs.

Next: [012-unreliability-causes-matrix.md](./012-unreliability-causes-matrix.md)
