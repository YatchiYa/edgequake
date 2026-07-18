# SPEC-063 — Architecture capacity assessment

**Status:** Active  
**Depends on:** SPEC-060/061/062 (latency SLOs + multi-major matrix)  
**Goal:** Separate **hard product caps**, **physics ceilings**, and **proven floors** so operators and docs never outrun evidence.

## Layers (inviolable)

| Layer | Meaning | How we treat claims |
|-------|---------|---------------------|
| Hard product caps | Code rejects / clamps | Always true regardless of hardware |
| Physics ceilings | Bytes/RAM/HNSW residency | Design envelope; needs RAM ≥ index |
| Proven floors | Measured under SLOs | Only these may be stated as “proven” |

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | Cost model (vectors, HNSW, pages→chunks, graph) |
| [`002-cap-catalog.md`](002-cap-catalog.md) | Enforced vs declared-unenforced vs soft-skip |
| [`003-operating-envelope.md`](003-operating-envelope.md) | Proven / supported / aspirational tables |
| [`004-proof-ladder.md`](004-proof-ladder.md) | L1/L2/L3 gates before raising FAQ claims |

## Commands

```bash
# Proven floor remasure (50k @1536)
make data-access-perf-matrix-prod

# Capacity ladder L1 (100k @1536) — pg18 release, not PR CI
make data-access-perf-capacity-ladder

# Close L1 cliff (halfvec / partial HNSW / GUC) — SPEC-064
make ann-scale-battle

# Cross-major latency compare (SPEC-062)
make compare-eq-perf
```

Artifacts: [`e2e/artifacts/`](e2e/artifacts/).

## Related

- Pins: [`edgequake/docker/extension-pins.sh`](../../edgequake/docker/extension-pins.sh)
- Data plane: [`docs/deep-dives/data-layer.md`](../../docs/deep-dives/data-layer.md)
- Prior storage-GB finding: [`specs/021-storage-study/06-first-principles/13-capacity-system-first-principles-assessment.md`](../021-storage-study/06-first-principles/13-capacity-system-first-principles-assessment.md)
