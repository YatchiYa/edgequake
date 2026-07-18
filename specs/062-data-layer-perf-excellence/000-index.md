# SPEC-062 — Data-layer performance excellence

**Status:** Remeasured (prod stress 2026-07-18)  
**Depends on:** SPEC-060 (stage SLOs), SPEC-061 (multi-major matrix)  
**Goal:** Beat the SPEC-061 SLO floor with measurement honesty, structural AGE write improvements, and ingest headroom — same floor on pg16/17/18.

## Waves

| Wave | Focus | Exit |
|------|-------|------|
| 0 | Measurement law | Every matrix gate emits `PERF_REPORT`; release matrix; sample hygiene; cross-major 2× gate |
| 1 | Kill agtype tax | Denormalized EDGE/Node id columns; pg16 edge upsert ≤1.3× pg18 |
| 2 | Ingest vector wall | HNSW insert p95 &lt;250ms @ matrix scale; halfvec greenfield documented |
| 3 | Query / stress | Filtered ANN in JSONL; stress ≤1.5× on pg17/18 |
| 4 | Ops playbooks | `data-layer.md` per-major; `make data-access-perf-matrix-release`; compare script |

## Pins

See [`edgequake/docker/extension-pins.sh`](../../edgequake/docker/extension-pins.sh): pg16 AGE≥1.6; pg17/18 AGE≥1.7; pgvector≥0.8.5.

## Commands

```bash
make data-access-perf-matrix              # debug (default)
make data-access-perf-matrix-release      # EDGEQUAKE_PERF_RELEASE=1
python3 scripts/compare_eq_perf_jsonl.py \
  specs/061-multi-version-data-access-perf/e2e/artifacts/eq-perf-pg18.jsonl \
  /tmp/eq-perf-pg18.jsonl
```
