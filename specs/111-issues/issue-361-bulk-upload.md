# issue-361 — Bulk upload excessively slow

**GH:** https://github.com/raphaelmansuy/edgequake/issues/361  
**Reported on:** **v0.12.11**  
**Status:** Capacity / expectation — not a confirmed logic bug on HEAD

## WHY

Bulk ingest latency must be measured against provider and concurrency law, not assumed broken.

## Code law (current)

- Pipeline is LLM + embed + graph write bound.
- Local vision concurrency intentionally capped (`pdf_processing.rs`).
- Pool / counter contention documented in [SPEC-090](../090-performance/).

## Fix plan

1. Collect: N files, sizes, provider, workers, wall clock, stage timings.
2. Compare SPEC-090 baselines.
3. Only then tune admission / concurrency — never unbounded parallel LLM.

## E2E

E2E-111-09 measurement only until SLO exists.
