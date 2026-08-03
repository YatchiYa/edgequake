# Reports schema

## Per-major directory (`pg16` / `pg17` / `pg18`)

| Artifact | Description |
| --- | --- |
| `verdict.md` | Human summary: status, AC table, timings, PG version |
| `verdict.json` | Machine-readable verdict |
| `soak.log` | Full harness log |
| `seed.env` | Tenant / workspace / doc id map |
| `migrate-dry-run.log` | Dry-run stdout |
| `migrate-refuse.log` | Migrate without `--confirm-drop` |
| `migrate-console.log` / `migrate-guard.log` | Advisor / guard |
| `migrate-confirm.log` | Confirm-drop tee |
| `head-api.log` | HEAD API process log |
| `pre-upgrade.dump.sha256` | SHA of dump (binary may be under `artifacts/`) |

Large `pre-upgrade.dump` files are stored under `artifacts/spec93-migration-assessment/<profile>/` and referenced by SHA from `verdict.md` to keep the git tree small.

## Matrix

| Artifact | Description |
| --- | --- |
| `matrix-summary.md` | Cross-major PASS/FAIL rollup |

Placeholder rows exist until the first formal run completes.
