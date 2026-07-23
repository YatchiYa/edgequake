# Cluster 07 — Implementation notes (Sprint 5 minimal)

> Defects: X-34, X-35, X-21 · Laws: LAW-3, LAW-8

## Acc@N gate (existing harness)

Do **not** quote Acc@5 as system accuracy. Publish Acc@N / F1@N curves; gate on larger N.

Primary smoke Acc gate (n=40 GraphRAG-Bench subset):

```bash
make bench001-smoke-acc
```

Related targets / scripts:

| Command | Role |
|---------|------|
| `make bench001-smoke-acc` | Acc-lift smoke @ n≈40 (daily gate candidate) |
| `make bench001-acc-backend` | Detached Acc-pinned backend for local runs |
| `./tools/bench001/scripts/run_p_ladder_acc.sh <ladder>` | Ladder Acc runs (identity / fusion ablations) |
| `./tools/bench001/scripts/run_f_ladder_acc.sh f1a\|f2a\|…` | Feature-ladder Acc |

Nightly / regression gates (Wave D landed):

- `nightly_golden_acc_gate` — **scores** all golden cases with deterministic mock-oracle keyword Acc/F1 (X-34); `count≥50` remains smoke; live LLM path is `nightly_golden_acc_gate_live_llm` (`#[ignore]`)
- `bench_acc_at_n_regression_gate` — loads `tests/fixtures/acc_at_n_floors.json`; Acc@40 measured fixture must stay ≥ `regression_floor_acc_at_40` (X-35)

```bash
cargo test -p edgequake-query --lib nightly_golden_acc_gate
cargo test -p edgequake-query --lib bench_acc_at_n_regression_gate
```

## ExplainTrace MVP (X-21 — landed)

`edgequake_query::ExplainTrace` is populated from `QueryStats` (`arms_run`, `sparse_outcome`, `query_intent`) and returned on engine `QueryResponse.explain`. The REST DTO mirrors it as `explain` on `/query` responses (`ExplainTraceDto`).

Operators can still use fusion env labels (`EDGEQUAKE_MIX_FUSION`, `EDGEQUAKE_SPARSE_FUSION=sparse_first|rrf`) and bench001 scorecards for deeper Acc@N analysis.

## Louvain hierarchy (D-54 — landed, opt-in)

Default remains flat Louvain phase-1. Set `EDGEQUAKE_LOUVAIN_HIERARCHY=1` to enable phase-2 community aggregation (multi-level). Evidence: `unit_louvain_hierarchy_levels` in `edgequake-storage` community tests.
