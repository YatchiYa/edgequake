# 01 — First Principles

## Axioms

1. **Admit ≠ complete.** HTTP 202 / track_id means durable enqueue, not searchable KG.
2. **Throughput is a min()** of worker count, tenant fairness, provider budget, vision jobs, extract fan-out, and embed async — not of files selected in the dropzone.
3. **Local LLM defaults protect the machine**, not maximize docs/min. Ollama parallel slots (`OLLAMA_NUM_PARALLEL`) are often 1; EdgeQuake mirrors that with near-serial ingest.
4. **Cloud/API providers buy parallelism with rate limits and $** — still O(chunks)×RTT per document.
5. **PDF convert is a quality-path tax** (pdfium + vision pages), orthogonal to format support (SPEC-121).
6. **Measure before mutate.** Stage timers + `queue-metrics` beat intuition.
7. **DRY:** one concurrency matrix feeds Makefile, docker-compose, FAQ, UI copy, tests.
8. **SOLID:** fairness, provider budget, PDF convert, extract, embed are separable adapters — do not fuse them into one “bulk mode” god-object.
9. **No unbounded fan-out.** Every concurrency raise has a budget, an escape hatch, and an e2e fairness gate.
10. **Partner trust** requires honest language: capacity law is not a silent bug.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-122-1** | Admit ≠ complete — 202 is transfer success, not searchability |
| **LAW-122-2** | Throughput = min(worker, tenant, provider, vision, extract, embed) |
| **LAW-122-3** | Local Ollama/LM Studio defaults are intentionally near-serial; raise only with VRAM/`OLLAMA_NUM_PARALLEL` headroom + `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY` when required |
| **LAW-122-4** | Cloud/Mistral may widen lanes but remain O(chunks)×RTT bound |
| **LAW-122-5** | Measure stages (queue-metrics + logs) before changing concurrency |
| **LAW-122-6** | PDF vision cost is a quality-path tax, not a format bug (cross-ref SPEC-121) |
| **LAW-122-7** | DRY: one concurrency SSOT table → Makefile, compose, FAQ, UI, tests |
| **LAW-122-8** | SOLID: fairness / provider budget / PDF convert / extract / embed stay separable |
| **LAW-122-9** | No unbounded fan-out; every raise has budget + e2e regression |
| **LAW-122-10** | E2E proves **throughput** and **fairness** (no query starvation / 429 storm) |

## Causal diagram (Five WHYs for #361/#365)

```ascii
  WHY “bulk upload excessively long”?
    → Wall clock ≈ Σ(stage_i) / effective_parallelism
  WHY effective_parallelism ≪ N files?
    → MAX_TASKS_PER_TENANT + extract/embed/vision clamps + provider budget
  WHY clamps exist?
    → Protect Ollama/VRAM/pool; avoid connection storms (SPEC-057/091)
  WHY Docker 0.24.1 still feels slow?
    → Wider lanes (tenant=6, extract=4) still LLM-bound; PDF vision pages dominate
  WHY partners call it a bug?
    → Expectation: batch ⇒ concurrent completion; UI hides queue physics
```

## Normative capacity policy

```ascii
  effective_parallelism =
    min(
      WORKER_THREADS,
      MAX_TASKS_PER_TENANT,          -- ingest lane
      PROVIDER_BUDGET or LOCAL_MAX_INFLIGHT,
      EDGEQUAKE_PDF_VISION_JOBS,     -- PDF convert only
      EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS,
      EDGEQUAKE_EMBED_MAX_ASYNC
    )

  peak_vision_inflight ≈ PDF_VISION_JOBS × PDF_CONCURRENCY

  searchable(doc) ⇔ Insert task status == Completed
  (PdfProcessing complete alone ≠ searchable)
```

## Cross-refs

- Matrix: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
- Ollama FAQ concurrent requests: https://docs.ollama.com/faq
