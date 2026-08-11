# 07 — Implementation Plan

## Principles

- **DRY:** one concurrency matrix → Makefile, compose, FAQ, UI, tests (LAW-122-7)
- **SOLID:** fairness / provider / PDF / extract / embed stay adapters (LAW-122-8)
- **First principles:** measure before mutate (LAW-122-5); admit ≠ complete (LAW-122-1)
- **Test first:** e2e proves throughput **and** fairness (LAW-122-10)
- **No unbounded fan-out** (LAW-122-9)

## Phase A — Truth & UX (P0)

1. Land this SPEC pack; fill [10-reproduction.md](10-reproduction.md) with Arm A/B/C numbers.
2. FAQ + performance-tuning + quick-start: publish concurrency SSOT + “why bulk feels slow”.
3. Measurement harness script (curl/WebUI-agnostic) recording admit_ms, t_first_complete, t_all_complete, queue-metrics snapshots.
4. UI/docs copy: bulk admit vs processing; local serial hint when applicable.
5. GitHub #361/#365 comment with SPEC-122 + measurements (do not close yet).

## Phase B — Provider-aware concurrency (P1)

**Gate:** Arm B shows headroom (low GPU/provider util, low park waiters) **or** partner SLO requires docs/min above measured baseline.

1. Align Docker PDF vision/extract defaults when provider is cloud API (document deltas).
2. Operator checklist for `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1` + `OLLAMA_NUM_PARALLEL`.
3. Regression: query latency under ingest; 429/503 handling; SPEC-090 store_contention.

## Phase C — PDF cost if proven (P2)

**Gate:** H2 or H3 accepted in [12-pdf-quality-hypothesis.md](12-pdf-quality-hypothesis.md).

1. Surface page-count ETA for PDF.
2. Optional cost controls (product-approved): concurrency hints, text-layer fast path if available.
3. Chunk-budget warning when markdown tokens explode vs source pages.

## Edge-case matrix

| ID    | Case                                                | Mitigation                      | Test         |
| -------| -----------------------------------------------------| ---------------------------------| --------------|
| EC-01 | N=1 baseline                                        | Control Arm C                   | T1           |
| EC-02 | N=3 WebUI                                           | Transfer≤3; ingest per profile  | T2           |
| EC-03 | N=10 text local                                     | Near-serial complete            | T3           |
| EC-04 | N=10 text Mistral/Docker                            | Wider overlap                   | T4           |
| EC-05 | Mixed PDF+text                                      | Dual-task PDF ordering          | T5           |
| EC-06 | Batch API 20 files                                  | Serial admit OK; process async  | T6           |
| EC-07 | Batch API 21 files                                  | Reject / clamp                  | T7           |
| EC-08 | Cancel mid-queue                                    | Remaining continue              | T8           |
| EC-09 | Provider 429                                        | Backoff; Failed or retry policy | T9           |
| EC-10 | Ollama 503 queue full                               | Surface error; no silent hang   | T10          |
| EC-11 | Tenant park under query                             | Interactive query not starved   | T11          |
| EC-12 | Huge PDF pages                                      | Vision concurrency clamp        | T12          |
| EC-13 | tenant=1 PDF convert+insert                         | Two serial phases               | T13          |
| EC-14 | Workspace isolation                                 | No cross-tenant claim           | existing     |
| EC-15 | Empty/corrupt PDF                                   | Fail convert, not hang          | SPEC-121 T9  |
| EC-16 | Duplicate hash                                      | duplicate_of path               | existing     |
| EC-17 | Raise local concurrency without OLLAMA_NUM_PARALLEL | Docs warn; optional test        | T14          |
| EC-18 | store_contention critical                           | `/ready` 503                    | SPEC-090/057 |
| EC-19 | Weak ETA                                            | Best-effort; never block admit  | UX           |
| EC-20 | Partner expects “ready on upload”                   | Copy + FAQ                      | T15          |

## Rollout

1. SPEC pack + reproduction measurements.
2. Phase A docs/UX/harness.
3. Phase B only if gate green.
4. Phase C only if H2/H3 green.
5. Close issues when [09-acceptance.md](09-acceptance.md) satisfied.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
