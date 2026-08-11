# 04 — Target Architecture

## Goal

Capacity-governed bulk ingest that is **honest**, **measurable**, and **tunable** — without collapsing fairness, provider budget, and PDF convert into one unsafe “bulk parallel” mode.

## Target control plane

```ascii
  ┌─────────────────────────────────────────────────────────┐
  │  ConcurrencyProfile (DRY SSOT)                          │
  │   local_ollama | docker_default | cloud_api             │
  │   → Makefile / compose / runtime assert same numbers    │
  └───────────────────────────┬─────────────────────────────┘
                              │
       ┌──────────────────────┼──────────────────────┐
       ▼                      ▼                      ▼
  FairnessLimiter      ProviderBudget          VisionBudget
  (tenant park)        (LLM inflight)          (PDF jobs×pages)
       │                      │                      │
       └──────────┬───────────┴──────────┬───────────┘
                  ▼                      ▼
            Task workers            Pipeline adapters
            (claim/lease)           extract | embed | convert
                  │
                  ▼
            Progress / ETA surface (LAW-122-1)
            admit_ms | queue_depth | stage | searchable_at
```

## SOLID boundaries

| Component | Responsibility | Must not |
|-----------|----------------|----------|
| Admit API | Validate, persist bytes, enqueue | Run LLM inline |
| FairnessLimiter | Cap concurrent ingest tasks/tenant | Know provider model names |
| ProviderBudget | Cap LLM/embed inflight | Own PDF page raster |
| PdfConvert adapter | Bytes→Markdown | Write KG entities |
| Extract/Embed adapters | Chunks→graph/vectors | Change tenant caps |
| Progress surface | Expose stage + ETA | Invent fake parallelism |

## DRY target

Single table (see [03-code-as-is.md](03-code-as-is.md)) is the only normative source. FAQ, UI strings, and tests **quote** it — they do not redefine it.

## Phased architecture changes

### Phase A (P0) — Truth surface

- Emit/display: admit duration, queue position/depth, current stage, estimated remaining (best-effort).
- FAQ: “why bulk is slow” + concurrency matrix + `queue-metrics`.
- Measurement harness script under `specs/122-implementation/scripts/` or `scripts/` that records Arm A/B timings.

### Phase B (P1) — Provider-aware profiles

- Document and optionally align Docker PDF vision defaults with cloud profile when `EDGEQUAKE_LLM_PROVIDER` is Mistral/OpenAI.
- Safe local raise only behind `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1` + ops checklist (`OLLAMA_NUM_PARALLEL`).
- Never remove tenant fairness.

### Phase C (P2) — PDF cost controls (gated)

- Only if H2/H3 accepted: page concurrency hints, skip-vision text layer option (if product allows), chunk-budget warnings.
- Must not weaken SPEC-121 format matrix.

## Anti-patterns

```ascii
  ✗ “Set extract=32 on Ollama laptop”
  ✗ Bypass MAX_TASKS_PER_TENANT for WebUI batches
  ✗ Mark document Searchable after PdfProcessing only
  ✗ Fourth concurrency table in marketing copy
  ✗ Parallelize claim_next without SPEC-090 awareness
```

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
