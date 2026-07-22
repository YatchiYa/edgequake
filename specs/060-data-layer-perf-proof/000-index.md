# SPEC-060 — Data-layer performance proof + complexity gates

Closes the gap between the SPEC-054 complexity catalog (documented) and **executable** proof per query/pipeline stage.

| Wave | Deliverable                                                      |
| ------| ------------------------------------------------------------------|
| 0    | Stage→complexity→proof matrix + FORBIDDEN contract               |
| 1    | Ingest stage + query arm Prometheus histograms                   |
| 2    | FTS / expand / ingest-stage / compensate / arm-wall Postgres e2e |
| 3    | Nightly `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1` + full suite        |
| 4    | Native-writes contract, FORBIDDEN CI, data-layer honesty         |

Cross-ref: [005-query-complexity-catalog](../054-fix-bugs-17/005-query-complexity-catalog.md), [003-budgets](../054-fix-bugs-17/003-performance-budgets-and-gates.md), [SPEC-059](../059-data-layer-integrity/000-index.md).

See [001-first-principles.md](001-first-principles.md), [002-stage-matrix.md](002-stage-matrix.md), [003-slos.md](003-slos.md).
