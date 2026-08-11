# issue-361 — Bulk upload excessively slow

**GH:** https://github.com/raphaelmansuy/edgequake/issues/361  
**Sibling:** https://github.com/raphaelmansuy/edgequake/issues/365  
**Reported on:** **v0.12.11** → confirmed **v0.24.1**  
**Status:** Capacity / expectation — measured on **v0.24.3** under SPEC-122

## WHY

Bulk ingest latency must be measured against provider and concurrency law, not assumed broken.

## Code law (current)

- Pipeline is LLM + embed + graph write bound.
- Local vision concurrency intentionally capped (`pdf_processing.rs`).
- Pool / counter contention documented in [SPEC-090](../090-performance/).
- Full First-Principles pack: [SPEC-122](../122-implementation/).

## Measurement (2026-08-11, SPEC-122)

| Arm | Provider | N | tenant | t_all_s | docs/min |
|-----|----------|---|--------|---------|----------|
| C | Ollama | 1 | 1 | 14.2 | 4.2 |
| A | Ollama | 5 | 1 | 59.2 | 5.1 |
| B | Mistral | 5 | 6 | 45.0 | 6.7 |

Admit ≪ processing on all arms. PDF 1-page vision convert ≈11.5 s (quality-path tax).

## Fix plan

1. ~~Collect timings~~ — done in SPEC-122 `10-reproduction.md`
2. Phase A: honest FAQ/UX/docs + harness — landed with SPEC-122
3. Phase B/C only if partner SLO requires — gated

## E2E

Harness: `specs/122-implementation/scripts/measure-bulk-ingest.py`
