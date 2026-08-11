# Lens 007 — Reliability Engineer

## Stake

Throughput changes without backpressure create cascading failure: Ollama queue 503, Mistral 429 storms, pool exhaustion, fairness collapse, zombie Processing rows.

## SLIs / SLOs (draft)

| SLI | Draft SLO (tune after measure) |
|-----|--------------------------------|
| Admit success rate | ≥99% under N≤20 |
| Time to admit N files | p95 < 30s for N≤20 small files |
| Queue visibility | queue-metrics always 200 when healthy |
| Fairness | Interactive query p95 not >2× baseline during ingest |
| Failure isolation | One Failed doc does not stall tenant forever |
| Ready signal | `/ready` 503 on critical store contention |

## Controls

1. Keep tenant park (no reclaim storm).
2. Provider budget / local inflight gates.
3. Cancel + lease expiry paths (SPEC-057).
4. Phase B raises require load test artifact in `measurements/`.
5. Chaos: kill vision mid-PDF; ensure Failed convert not silent Pending.

## Cross-refs

- Laws LAW-122-9/10: [../01-first-principles.md](../01-first-principles.md)
- DB lens: [003-database.md](003-database.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
