# Measurements (SPEC-107)

Drop proof runs here after e2e gates.

| File | Content | Result (2026-08-04) |
|------|---------|---------------------|
| `e2e107-storage-inspector.txt` | earlier SPEC-107 run | 5/5 (pre-R2) |
| `e2e107-contract-spec104.txt` | earlier SPEC-107 run | 12/12 (pre-R2) |
| `e2e107-r2-storage-inspector.txt` | `--features postgres --lib storage_inspector` | **6/6 ok** incl. `e2e_107_r2_source_count_bounds_ssot` |
| `e2e107-r2-contract.txt` | `--features postgres --test contract_spec104_datalayer` | **13/13 ok** incl. `e2e_107_r2_inv_c_chunks_by_batch_limit` |
| `e2e107-r2-issue331-336.txt` | `e2e_issue331_*` + `e2e_issue336_*` | **3+6 ok** (PG cases skip without `DATABASE_URL`) |

V22 “errors present” evidence stays in [SPEC-104 measurements](../104-fix-datalayer/measurements/).
