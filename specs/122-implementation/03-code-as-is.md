# 03 — Code As-Is

## Concurrency matrix (SSOT snapshot)

| Knob | WebUI | make local (Ollama) | Docker compose | make cloud (OpenAI/Mistral key path) |
|------|-------|---------------------|----------------|--------------------------------------|
| Transfer / admit parallel | **3** | n/a | n/a | n/a |
| `WORKER_THREADS` | — | **2** | **8** | **16** |
| `MAX_TASKS_PER_TENANT` | — | **1** | **6** | **12** |
| `EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS` | — | **1** | **4** | **32** |
| `EDGEQUAKE_EMBED_MAX_ASYNC` | — | **1** | (unset→code default) | **8** |
| `EDGEQUAKE_PDF_VISION_JOBS` | — | **1** | (unset→code default ~2) | **4** |
| `EDGEQUAKE_PDF_CONCURRENCY` | — | **1** | (unset→profile) | **4** |
| `EDGEQUAKE_PROVIDER_BUDGET` | — | **1** | (unset) | **0** (legacy hard-cap off) |
| Batch files / request | — | — | — | API max **20** |

Escape hatch: `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1` lifts local Makefile-style clamps when explicitly set.

## Control flow (as-is)

```ascii
  Dropzone multi-select
       │
       ▼
  createBoundedExecutor(3)
       │  Promise.all per file
       ├─ .pdf  → POST /documents/pdf     → TaskType::PdfProcessing
       ├─ text  → POST /documents…        → TaskType::Insert
       └─ image → POST /documents/upload  → Insert (VLM)
              │
              ▼
  Postgres task row + wake
              │
              ▼
  Workers claim_next (SKIP LOCKED)
              │
              ├─ if tenant at MAX_TASKS_PER_TENANT → park (fairness)
              │
              ├─ PdfProcessing
              │     acquire PdfVisionSemaphore
              │     page fan-out ≤ PDF_CONCURRENCY
              │     on success → enqueue Insert
              │
              └─ Insert
                    chunk → extract(sem) → embed(buffer_unordered)
                    → graph/KV writes → Completed
```

## Hotspot sources

| Stage | File | Notes |
|-------|------|-------|
| WebUI bound | `bounded-file-upload.ts` | `MAX_CONCURRENT_FILE_UPLOADS = 3` |
| Batch API | `batch_upload.rs` | Serial `for` over files |
| Workers | `edgequake-tasks/src/worker.rs` | One claimed task occupies one worker for stage wall-clock |
| PDF profile | `pdf_processing.rs` `compute_safe_pdf_resource_profile` | Local vs cloud page concurrency |
| Extract | `extraction.rs` | `Semaphore` + `buffer_unordered` |
| Embed | `embeddings.rs` | `LOCAL_EMBED_MAX_ASYNC = 1` |
| Make SSOT | `Makefile` | Provider-aware `ifeq` Ollama vs cloud |
| Compose SSOT | `docker-compose.yml` | Reporter-like Docker topology |

## Implications for #361/#365

1. **Local `make dev`:** near-serial completion is **by design** (LAW-122-3).
2. **Docker v0.24.1:** not serial at tenant=6 — still feels slow if PDF vision + extract dominate.
3. **WebUI “batch”** is transfer parallelism only — not a multiplexed pipeline job.
4. **PDF** pays convert **then** insert; under tenant=1 that is two serial slots.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Matrix: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
