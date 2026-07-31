# 03 — Execution Checklist

Use this when running the formal matrix.

## Before

- [x] Docker daemon healthy (`docker info`)
- [x] GHCR reachable (`docker pull ghcr.io/raphaelmansuy/edgequake:0.22.0`)
- [x] Pull postgres tags: `0.22.0-pg16`, `0.22.0-pg17`, `0.22.0-pg18`
- [x] Workspace compiles: `cd edgequake && cargo build -p edgequake --features postgres --bin edgequake`
- [x] No conflicting soak compose projects (`docker compose -p spec93soak-pg16 ps` empty)

## Run

- [x] `make spec93-migration-assessment`  
  **or** sequential: `make spec93-migration-assessment-pg16` then pg17 then pg18
- [x] Confirm each major wrote `reports/pgN/verdict.md` with `status: GREEN`
- [x] Confirm `reports/matrix-summary.md` overall PASS
- [x] Confirm dump SHA recorded (binary under `artifacts/spec93-migration-assessment/` if not inlined)

## After

- [x] Spot-check one `migrate-confirm.log` for `applied 125` and `KV store dropped`
- [x] Spot-check `_sqlx_migrations` max ≥ 137 in one major's soak log
- [x] Update RM-AC-13 in SPEC-091 assessment if matrix GREEN
- [x] Link ops runbook to this pack

## Abort / retry

- On FAIL: leave `SPEC091_SOAK_KEEP=1` for debugging, inspect `soak.log` + `head-api.log`
- Retry single major: `make spec93-migration-assessment-pg17`
- Force re-pull: unset `SPEC091_SOAK_SKIP_PULL`

## Formal run record

| Field | Value |
| --- | --- |
| Finished | 2026-07-31T11:10:24Z |
| Overall | **PASS** |
| Evidence | [reports/matrix-summary.md](reports/matrix-summary.md) |
