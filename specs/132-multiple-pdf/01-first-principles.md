# 01 — First Principles (LAW-132)

## Domain

Multi-PDF ingest has **two success planes**:

```ascii
  Plane A — Admit (upload done)
            HTTP 2xx + durable pdf/document + task row + task_id
            Client: row leaves “transferring” with track_id

  Plane B — Process (searchable)
            Convert → chunk → embed → extract → graph
            Bound by vision / tenant / LLM capacity (SPEC-122)
```

Operators reporting #378 speak Plane A. Partners reporting #361 speak Plane B. Mixing them burns trust.

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-132-1 | **Admit ≠ process** — “uploaded” means Plane A only | Honest vocabulary |
| LAW-132-2 | **HTTP never blocks on wake** — persist-first; wake is best-effort / timeout; hydrate recovers | Close F-091-19 / LD-12 |
| LAW-132-3 | **Per-file isolation** — one hung/failed PDF must not freeze siblings forever without error | Cap-3 executor honesty |
| LAW-132-4 | **Progress identity** — progress key = server `task_id`; client `batchTrackId` is correlation only | SPEC-054 / #300 |
| LAW-132-5 | **PDF routes only** — `/documents/pdf` or `/pdf/batch`; never `/upload/batch` for PDFs | SPEC-123 |
| LAW-132-6 | **Capacity honesty** — slow convert/extract uses SPEC-122 language, not “upload failed” | Separate #361 |
| LAW-132-7 | **Unfakable proof** — multi-PDF e2e + admit-non-block test; no Acc throughput claim | Honest acceptance |
| LAW-132-8 | **DRY enqueue** — one delivery SSOT for wake; do not special-case PDF-only | SOLID SRP |
| LAW-132-9 | **WebUI stays N× single PDF** — keep GH-350 path; `/pdf/batch` remains API/SDK | Avoid dual stacks |

## Capacity constants (as-is anchors)

```ascii
  Client transfer concurrency     = 3
  Wake channel capacity           = 100
  Max upload body (single HTTP)   = 50 MiB
  Max batch file count            = 20 (env clamp 1–500)
  Vision jobs concurrent (default)= 2
  Docker MAX_TASKS_PER_TENANT     = 6
```

## Cross-refs

- Why: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Parent queue: [../091-simplify-data-layer/](../091-simplify-data-layer/)
- Parent capacity: [../122-implementation/](../122-implementation/)
