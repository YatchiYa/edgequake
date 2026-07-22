# SPEC-061 — Inviolable multi-version data-access performance matrix

Makes every catalog **OK** DataAccess path measurable and gated on **PG16 / PG17 / PG18** with pinned AGE + pgvector + FTS.

| Wave | Deliverable                                                                     |
| ------| ---------------------------------------------------------------------------------|
| 0    | Op × major × proof matrix SSOT                                                  |
| 1    | Shared Rust harness + `run_data_access_perf_matrix.sh`                          |
| 2    | Missing OK-path gates (KV, unfiltered ANN, edges, degrees, PG QueryEngine arms) |
| 3    | Concurrent stress (ANN / FTS / expand / Mix)                                    |
| 4    | Nightly CI matrix + image build + JSON artifacts                                |
| 5    | Docs / catalog markers / Makefile                                               |

## Pins (SSOT)

| Profile | PG | pgvector | AGE |
|---------|----|----------|-----|
| pg16 | 16 | ≥0.8.5 | ≥1.6.0 |
| pg17 | 17 | ≥0.8.5 | ≥1.7.0 |
| pg18 | 18 | ≥0.8.5 | ≥1.7.0 |

Source: [`edgequake/docker/extension-pins.sh`](../../edgequake/docker/extension-pins.sh).

## Relationship

- **SPEC-042** — capability battle (features exist)
- **SPEC-046 OPS-17** — pin smoke (versions pinned)
- **SPEC-060** — single-major p95/EXPLAIN proof (ladder base)
- **SPEC-061** — same proofs × all majors + stress + artifacts

Cross-ref: [001-first-principles](001-first-principles.md), [002-op-matrix](002-op-matrix.md), [003-slos](003-slos.md).
