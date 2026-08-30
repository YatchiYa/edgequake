# issue-361 — Bulk upload excessively slow

**GH:** https://github.com/raphaelmansuy/edgequake/issues/361  
**Sibling:** https://github.com/raphaelmansuy/edgequake/issues/365  
**Reported on:** **v0.12.11** → confirmed **v0.24.1**  
**Status:** **Closed 2026-08-30** as capacity / expectation (not a correctness defect). Re-measured on **v0.26.3**.

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

## HEAD re-measure (2026-08-30, v0.26.3, Docker-like + Mistral)

| Arm | N | admit_s | t_all_s | docs/min | tenant |
|-----|---|---------|---------|----------|--------|
| C | 1 | 0.099 | 12.276 | 4.888 | 6 |
| D | 5 | 0.186 | 51.275 | 5.851 | 6 |

Artifact: [`../122-implementation/measurements/20260830-summary.json`](../122-implementation/measurements/20260830-summary.json).  
Playwright: `spec122-admit-honesty` green.  
Harness: `make measure-bulk-ingest ARM=D N=5`.

## Fix plan

1. ~~Collect timings~~ — done in SPEC-122 `10-reproduction.md`
2. Phase A: honest FAQ/UX/docs + harness — landed with SPEC-122
3. ~~Close as capacity~~ — 2026-08-30 (no partner SLO; honesty + measure)
4. Phase B/C only if partner proposes a concrete docs/min SLO — still gated

## E2E

```bash
make measure-bulk-ingest ARM=D N=5
# WebUI honesty:
# EQ_BACKEND_URL=… E2E_LIVE_STACK=1 PLAYWRIGHT_BASE_URL=… \
#   pnpm exec playwright test e2e/spec122-admit-honesty.spec.ts
```
