# 11 — Honest Assessment

## What the reporter got right

1. Bulk ingest **does** take a long wall clock for N documents.
2. Documents remain non-searchable while Processing — productivity impact is real.
3. Expectation of concurrent completion is reasonable for a “batch upload” UX label.
4. Re-confirmed on **v0.24.1** Docker — not only ancient v0.12.11.

## What is not a logic bug

1. Local near-serial ingest under Ollama clamps is **intentional** (VRAM / `OLLAMA_NUM_PARALLEL`).
2. WebUI uploading 3-at-a-time is **not** the main processing bottleneck.
3. HTTP 202 “success” while Processing continues is **by design** (async pipeline).
4. Unbounded “parallelize everything” would likely worsen reliability (429, OOM, SPEC-090 contention).

## Residual risks

| Risk | Mitigation |
|------|------------|
| Partner rejects capacity explanation | Ship Phase A honesty + optional Phase B SLO |
| Docker+Mistral still slow (LLM-bound) | Document docs/min; tune extract within rate limits |
| PDF vision dominates | Phase C gated on H2 |
| Raising concurrency blindly | LAW-122-9 + fairness e2e |
| Docs drift vs Makefile | DRY matrix + tests T10 |

## Confidence

| Claim | Confidence |
|-------|------------|
| Capacity/expectation classification | High (code + prior #361 comments) |
| Local H1 serial lane | High a priori; confirm in Arm A |
| Docker partner path | Medium until Arm B numbers |
| PDF quality as root cause | Low as root; Medium as amplifier |

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
