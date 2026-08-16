# Lens 006 — AI Engineer

## Stake

WebUI sets `enable_vision: true` on every PDF admit. Multi-PDF therefore contends for the **vision job semaphore** (default ≈2). That starves Plane B and feels like “upload stuck” even when Plane A succeeded.

## First principles

```ascii
  Vision convert = LLM/VLM bound (quality path)
       │
       ▼
  Process-wide PDF_VISION_JOBS concurrent ≈ 2
       │
       ▼
  N PDFs admitted OK → only ≤2 convert at once
       │
       ▼
  Honest UX: “Queued for conversion” ≠ “Upload failed”
```

## Operator knobs (document, do not unbounded-raise for #378)

- Vision host reachability from Docker (`OLLAMA_HOST` / OpenAI vision)
- `EDGEQUAKE_*` vision concurrency envs (existing budget.rs)
- Tenant fairness `MAX_TASKS_PER_TENANT`

## Eval

- Arm D reproduction: vision down → admit still returns `task_id` (LAW-132-1).
- Do not claim Acc/quality improvements from SPEC-132.

## Cross-refs

- Reproduction: [../12-reproduction.md](../12-reproduction.md)
- SPEC-122: [../../122-implementation/](../../122-implementation/)
