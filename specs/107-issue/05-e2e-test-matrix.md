# 05 — E2E Test Matrix (SPEC-107)

Absence proofs for partner error classes on HEAD. **Reuse** SPEC-104 contracts; do not duplicate product fixtures.

| ID | Asserts | Mechanism | Gate |
|----|---------|-----------|------|
| **E2E-107-01** | No INV-D2 `workspaces.id` probe | Source: `WHERE workspace_id::text`; no `WHERE id::text = $1` | `contract_spec104_datalayer::e2e_104_01_*` |
| **E2E-107-02** | Inspector graph ≠ `edgequake` | `InspectorConfig::default().graph_name == eq_eq_default_graph` | `storage_inspector::e2e_104_02_*` + contract |
| **E2E-107-03** | INV-03 dual-read + LogOnly repair | Source dual-read; unit `e2e_107_03_inv03_logonly_repair` | lib + contract |
| **E2E-107-04** | Tenant slug race → 200/409 not raw 23505 | Idempotent create / Conflict | `contract_spec104` + workspace_service tests |
| **E2E-107-R2-01** | INV-C chunks by `SOURCE_PREFIX_BATCH_LIMIT` | Source: `chunks(batch_limit)` + `one_batch` | `e2e_107_r2_inv_c_chunks_by_batch_limit` |
| **E2E-107-R2-02** | Bounds SSOT 32 / 300 (no raise) | Unit `e2e_107_r2_source_count_bounds_ssot` | lib `--features postgres` |
| Existing | GIN locality + list boundedness | `e2e_issue331_*`, `e2e_issue336_*` | edgequake-storage |

## Commands

```bash
cargo test -p edgequake-api --features postgres --test contract_spec104_datalayer -- --nocapture
cargo test -p edgequake-api --features postgres --lib storage_inspector -- --nocapture
# optional live PG:
# DATABASE_URL=postgres://... cargo test -p edgequake-api --features postgres --test contract_spec104_datalayer -- --nocapture
```

V22 repro (error **present**): [SPEC-104 11-v22-docker-repro](../104-fix-datalayer/11-v22-docker-repro.md) — do not rebuild.

## Measurements

Capture under [measurements/](measurements/). See [measurements/README.md](measurements/README.md).
